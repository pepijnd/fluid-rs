#![allow(dead_code)]

pub mod spatial;
pub mod spatial_vec;
use rand::{thread_rng, Rng};

struct Particles {
    x: Vec<f32>,
    y: Vec<f32>,
    oldx: Vec<f32>,
    oldy: Vec<f32>,
    vx: Vec<f32>,
    vy: Vec<f32>,
    p: Vec<f32>,
    pnear: Vec<f32>,
    g: Vec<f32>,
}

impl Particles {
    fn new(count: usize) -> Particles {
        Particles {
            x: Vec::with_capacity(count),
            y: Vec::with_capacity(count),
            oldx: Vec::with_capacity(count),
            oldy: Vec::with_capacity(count),
            vx: Vec::with_capacity(count),
            vy: Vec::with_capacity(count),
            p: Vec::with_capacity(count),
            pnear: Vec::with_capacity(count),
            g: Vec::with_capacity(count),
        }
    }

    fn push(&mut self, x: f32, y: f32) {
        self.x.push(x);
        self.y.push(y);
        self.oldx.push(0f32);
        self.oldy.push(0f32);
        self.vx.push(0f32);
        self.vy.push(0f32);
        self.p.push(0f32);
        self.pnear.push(0f32);
        self.g.push(0f32);
    }
}

pub struct FluidDomain {
    width: usize,
    height: usize,
}

impl FluidDomain {
    pub fn new(width: usize, height: usize) -> FluidDomain {
        FluidDomain { width, height }
    }
}

pub struct SimulationSettings {
    time_scale: u32,
    gravity: (f32, f32),
    interaction_radius: f32,
    stiffness: f32,
    stiffness_near: f32,
    rest_density: f32,
    time_delta: f32,
}

impl SimulationSettings {
    pub fn new(time_scale: u32, gravity: (f32, f32), interaction_radius: f32, stiffness: f32, stiffness_near: f32, rest_density: f32) -> SimulationSettings {
        SimulationSettings {
            time_scale,
            gravity,
            interaction_radius,
            stiffness,
            stiffness_near,
            rest_density,
            time_delta: 1f32 / (time_scale as f32),
        }
    }
}

pub struct Fluid {
    particles: Particles,
    particle_count: usize,
    domain: FluidDomain,
    simulation_settings: SimulationSettings,
}

impl Fluid {
    pub fn new(particle_count: usize, domain: FluidDomain, simulation_settings: SimulationSettings) -> Fluid {
        let mut particles = Particles::new(particle_count);

        let mut rng = thread_rng();
        for _ in 0..particle_count {
            let x = rng.gen_range(0, domain.width) as f32;
            let y = rng.gen_range(0, domain.height) as f32;
            particles.push(x, y);
        }

        Fluid {
            particles,
            particle_count,
            domain,
            simulation_settings,
        }
    }

    pub fn get_particles(&self) -> (&Vec<f32>, &Vec<f32>) {
        (&self.particles.x, &self.particles.y)
    }

    pub fn update(&mut self) {
        // pass 1
        for i in 0..self.particle_count {
            self.particles.oldx[i] = self.particles.x[i];
            self.particles.oldy[i] = self.particles.y[i];
            self.apply_global_forces(i);

            self.particles.x[i] += self.particles.vx[i] * self.simulation_settings.time_delta;
            self.particles.y[i] += self.particles.vy[i] * self.simulation_settings.time_delta;
        }

        // pass 2
        for i in 0..self.particle_count {
            let neighbours = self.calc_neighbours(i);
            self.update_pressure(i, &neighbours);
            self.relax(i, &neighbours)
        }

        // pass 3
        for i in 0..self.particle_count {
            self.contain_particle(i);
            self.calc_velocity(i);
        }
    }

    fn apply_global_forces(&mut self, particle: usize) {
        let time_delta = self.simulation_settings.time_delta;
        let force = self.simulation_settings.gravity;

        self.particles.vx[particle] += force.0 * time_delta;
        self.particles.vy[particle] += force.1 * time_delta;
    }

    fn calc_neighbours(&mut self, particle: usize) -> Vec<usize> {
        let i = particle;
        let mut neighbours: Vec<usize> = Vec::new();
        for k in 0..self.particle_count {
            if i == k { continue; };
            let gradient = self.gradient(i, k);
            if gradient < 0.01 { continue; }
            self.particles.g[k] = gradient;
            neighbours.push(k)
        }
        neighbours
    }

    fn gradient(&self, particle: usize, neighbour: usize) -> f32 {
        let i = particle;
        let k = neighbour;
        let particle = (self.particles.x[i], self.particles.y[i]);
        let neighbour = (self.particles.x[k], self.particles.y[k]);

        let diff = ((particle.0 - neighbour.0).abs(), (particle.1 - neighbour.1).abs());
        let lsq = diff.0.powi(2) + diff.1.powi(2);

        if lsq > self.simulation_settings.interaction_radius.powi(2) { return 0f32; };
        let distance = lsq.sqrt();

        1f32 - (distance / self.simulation_settings.interaction_radius)
    }

    fn update_pressure(&mut self, particle: usize, neighbors: &Vec<usize>) {
        let i = particle;
        let mut density = 0f32;
        let mut near_density = 0f32;

        for n in neighbors {
            let n = *n;
            let g = self.particles.g[n];
            density += g.powi(2);
            near_density += g.powi(3);
        }

        let stiffness = self.simulation_settings.stiffness;
        let rest_density = self.simulation_settings.rest_density;
        let stiffness_near = self.simulation_settings.stiffness_near;
        self.particles.p[i] = stiffness * (density - rest_density);
        self.particles.pnear[i] = stiffness_near * near_density;
    }

    fn relax(&mut self, particle: usize, neighbours: &Vec<usize>) {
        let i = particle;
        let time_delta = self.simulation_settings.time_delta;
        let pos = (self.particles.x[i], self.particles.y[i]);

        for n in neighbours {
            let n = *n;
            let g = self.particles.g[n];

            let n_pos = (self.particles.x[n], self.particles.y[n]);
            let magnitude = self.particles.p[i] * g + self.particles.pnear[i] * g.powi(2);
            let pos_dif = (pos.0 - n_pos.0, pos.1 - n_pos.1);
            let dir = Self::unit_approx(pos_dif);
            let force = (dir.0 * magnitude, dir.1 * magnitude);
            let d = (force.0 * time_delta.powi(2), force.1 * time_delta.powi(2));

            self.particles.x[i] += d.0 * -0.5;
            self.particles.y[i] += d.1 * -0.5;

            self.particles.x[n] += d.0 * 0.5;
            self.particles.y[n] += d.1 * 0.5;
        }
    }

    fn unit_approx(diff: (f32, f32)) -> (f32, f32) {
        let tangent = diff.1.atan2(diff.0);
        (
            tangent.sin(),
            tangent.cos()
        )
    }

    fn contain_particle(&mut self, particle: usize) {
        let i = particle;
        let mut pos = (self.particles.x[i], self.particles.y[i]);

        if pos.0 < 0f32 { pos.0 = 0f32 }
        if pos.0 > self.domain.width as f32 { pos.0 = self.domain.width as f32 }
        if pos.1 < 0f32 { pos.1 = 0f32 }
        if pos.1 > self.domain.height as f32 { pos.0 = self.domain.height as f32 }

        self.particles.x[i] = pos.0;
        self.particles.y[i] = pos.1;
    }

    fn calc_velocity(&mut self, particle: usize) {
        let i = particle;
        let time_scale = self.simulation_settings.time_scale;

        let pos = (self.particles.x[i], self.particles.y[i]);
        let old = (self.particles.oldx[i], self.particles.oldy[i]);
        let v = ((pos.0 - old.0) * time_scale as f32, (pos.1 - old.1) * time_scale as f32);

        self.particles.vx[i] = v.0;
        self.particles.vy[i] = v.1;
    }
}
use wasm_bindgen::prelude::*;

mod fluid;

pub use fluid::{Fluid, FluidDomain, SimulationSettings};

#[wasm_bindgen]
pub struct Particles {
    x: Box<[f32]>,
    y: Box<[f32]>,
}

#[wasm_bindgen]
impl Particles {
    #[wasm_bindgen]
    pub fn get_x(&self) -> Box<[f32]> {
        self.x.clone()
    }

    #[wasm_bindgen]
    pub fn get_y(&self) -> Box<[f32]> {
        self.y.clone()
    }

    pub fn new(x: Box<[f32]>, y: Box<[f32]>) -> Particles {
        Particles { x, y }
    }
}

#[wasm_bindgen]
pub struct JSFluid {
    fluid: Fluid
}

#[wasm_bindgen]
impl JSFluid {
    #[wasm_bindgen(constructor)]
    pub fn new(particle_count: usize,
               width: usize, height: usize,
               time_scale: u32,
               gravity: f32, interaction_radius: f32,
               stiffness: f32, stiffness_near: f32, rest_density: f32,
    ) -> JSFluid {
        let fluid = Fluid::new(
            particle_count,
            FluidDomain::new(width, height),
            SimulationSettings::new(
                time_scale,
                (0f32, gravity),
                interaction_radius,
                stiffness,
                stiffness_near,
                rest_density,
            ));
        JSFluid { fluid }
    }

    #[wasm_bindgen]
    pub fn update(&mut self) {
        self.fluid.update();
    }

    #[wasm_bindgen]
    pub fn get_particles(&self) -> Particles {
        let particles = self.fluid.get_particles();
        let x = particles.0.clone();
        let y = particles.1.clone();
        Particles::new(x.into_boxed_slice(), y.into_boxed_slice())
    }
}
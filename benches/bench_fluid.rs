#![feature(test)]

extern crate test;
mod first {
    use test::{Bencher, black_box};
    use fluid_rust::{Fluid, FluidDomain, SimulationSettings};

    #[bench]
    fn bench_fluid(b: &mut Bencher) {
        b.iter(|| {
            let mut fluid = black_box(Fluid::new(
                1000,
                FluidDomain::new(1000, 1000),
                SimulationSettings::new(
                    60,
                    (0f32, 1f32),
                    9f32,
                    0.5f32,
                    0.5f32,
                    100f32
                )));
            for _ in 0..5 {
                fluid.update();
            }
        })
    }
}

mod second {
    use test::{Bencher, black_box};
    use fluid_rust::spatial::{Fluid, FluidDomain, SimulationSettings};

    #[bench]
    fn bench_fluid(b: &mut Bencher) {
        b.iter(|| {
            let mut fluid = black_box(Fluid::new(
                1000,
                FluidDomain::new(1000, 1000),
                SimulationSettings::new(
                    60,
                    (0f32, 1f32),
                    9f32,
                    0.5f32,
                    0.5f32,
                    100f32
                )));
            for _ in 0..5 {
                fluid.update();
            }
        })
    }
}

mod third {
    use test::{Bencher, black_box};
    use fluid_rust::spatial_vec::{Fluid, FluidDomain, SimulationSettings};

    #[bench]
    fn bench_fluid(b: &mut Bencher) {
        b.iter(|| {
            let mut fluid = black_box(Fluid::new(
                1000,
                FluidDomain::new(1000, 1000),
                SimulationSettings::new(
                    60,
                    (0f32, 1f32),
                    9f32,
                    0.5f32,
                    0.5f32,
                    100f32
                )));
            for _ in 0..5 {
                fluid.update();
            }
        })
    }
}
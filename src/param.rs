use rand::Rng;

use crate::context::RunContext;

pub enum Param {
    Constant(f32),
    RandFlat(f32, f32),  // min, max
    RandNorm(f32, f32),  // mean, stddev
    Changing(f32, f32),  // start, velocity
}

impl Param {
    pub fn eval(&self, ctx: &RunContext, age: f32) -> f32 {
        match self {
            Param::Constant(val) => *val,
            Param::RandFlat(min, max) => {
                let mut rng = ctx.rng.borrow_mut();
                rng.gen_range(*min..*max)
            },
            Param::RandNorm(mean, stdev) => {
                let mut rng = ctx.rng.borrow_mut();
                let val = rng.gen_range(0.0..1.0) + rng.gen_range(0.0..1.0) + rng.gen_range(0.0..1.0) - 1.5;
                (val * stdev / 0.522) + mean
            },
            Param::Changing(start, velocity) => {
                start + age * velocity
            }
        }
    }

}
use rand::Rng;

use crate::context::RunContext;

pub enum Param {
    Constant(f32),
    RandFlat(f32, f32),  // min, max
    RandNorm(f32, f32),  // mean, stddev
    Changing(f32, f32),  // start, velocity
}

pub fn eval(param: &Param, ctx: &RunContext, age: f32) -> f32 {
    match param {
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
        _ => 0.0, //###
    }
}


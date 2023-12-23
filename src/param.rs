use rand::Rng;

use crate::context::RunContext;

#[derive(Clone)]
pub enum Param {
    Constant(f32),
    RandFlat(f32, f32),  // min, max
    RandNorm(f32, f32),  // mean, stddev
    Changing(f32, f32),  // start, velocity

    Quote(Box<Param>),
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
            Param::Quote(_) => {
                panic!("eval Quote");
            }
        }
    }

    pub fn min(&self, _ctx: &RunContext, age: f32) -> Option<f32> {
        match self {
            Param::Constant(val) => Some(*val),
            Param::RandFlat(min, _max) => Some(*min),
            Param::RandNorm(mean, stdev) => {
                Some((-1.5 * stdev / 0.522) + mean)
            },
            Param::Changing(start, velocity) => {
                if *velocity < 0.0 {
                    None
                }
                else {
                    Some(start + age * velocity)
                }
            },
            Param::Quote(_) => {
                panic!("eval Quote");
            }
        }
    }

    pub fn max(&self, _ctx: &RunContext, age: f32) -> Option<f32> {
        match self {
            Param::Constant(val) => Some(*val),
            Param::RandFlat(_min, max) => Some(*max),
            Param::RandNorm(mean, stdev) => {
                Some((1.5 * stdev / 0.522) + mean)
            },
            Param::Changing(start, velocity) => {
                if *velocity > 0.0 {
                    None
                }
                else {
                    Some(start + age * velocity)
                }
            },
            Param::Quote(_) => {
                panic!("eval Quote");
            }
        }
    }

    pub fn resolve(&self, ctx: &RunContext, age: f32) -> Param {
        match self {
            Param::Quote(param) => *param.clone(),
            Param::Constant(_) => self.clone(),
            _ => Param::Constant(self.eval(ctx, age))
        }
    }

}

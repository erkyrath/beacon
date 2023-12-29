use std::fmt;
use rand::Rng;

use crate::waves::WaveShape;
use crate::context::RunContext;

// To think about:
// Params containing params? RandFlat(0, Changing())
// Params with state? (RandomWalk?) Currently not possible.
// Params which depend on Ops?
// (In an ideal universe, Params would be unified with Ops anyway.)

#[derive(Clone)]
pub enum Param {
    Constant(f32),
    RandFlat(f32, f32),  // min, max
    RandNorm(f32, f32),  // mean, stddev
    Changing(f32, f32),  // start, velocity
    Wave(WaveShape, f32, f32, f32), // shape, min, max, duration
    WaveCycle(WaveShape, f32, f32, f32), // shape, min, max, period

    Quote(Box<Param>),
}

impl fmt::Debug for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Param::Constant(val) => write!(f, "Constant({})", val),
            Param::RandFlat(min, max) => write!(f, "RandFlat(min={}, max={})", min, max),
            Param::RandNorm(mean, stdev) => write!(f, "RandNorm(mean={}, stdev={})", mean, stdev),
            Param::Changing(start, velocity) => write!(f, "Changing(start={}, velocity={})", start, velocity),
            Param::Wave(shape, min, max, duration) => write!(f, "Wave(shape={:?}, min={}, max={}, duration={})", shape, min, max, duration),
            Param::WaveCycle(shape, min, max, period) => write!(f, "WaveCycle(shape={:?}, min={}, max={}, period={})", shape, min, max, period),
            Param::Quote(param) => write!(f, "Quote({:?})", *param)
        }
    }
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
            },
            Param::Wave(shape, min, max, dur) => {
                shape.sample(age/dur) * (max-min) + min
            },
            Param::WaveCycle(shape, min, max, period) => {
                shape.sample((age/period).rem_euclid(1.0)) * (max-min) + min
            },
            Param::Quote(_) => {
                panic!("eval Quote");
            },
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
            Param::Wave(_shape, min, _max, _period) => Some(*min),
            Param::WaveCycle(_shape, min, _max, _period) => Some(*min),
            Param::Quote(_) => {
                panic!("eval Quote");
            },
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
            Param::Wave(_shape, _min, max, _period) => Some(*max),
            Param::WaveCycle(_shape, _min, max, _period) => Some(*max),
            Param::Quote(_) => {
                panic!("eval Quote");
            },
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

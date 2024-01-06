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
pub enum ParamDef {
    Constant(f32),
    RandFlat(f32, f32),  // min, max
    RandNorm(f32, f32),  // mean, stddev
    Changing(f32, f32),  // start, velocity
    Wave(WaveShape, f32, f32, f32), // shape, min, max, duration
    WaveCycle(WaveShape, f32, f32, f32, f32), // shape, min, max, period, offset

    Quote(Box<Param>),
}

#[derive(Clone)]
pub struct EParam {
    def: ParamDef,
    args: Vec<Param>,
}

#[derive(Clone)]
pub enum Param {
    Const(f32),
    Param(Box<EParam>),
}

impl fmt::Debug for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Param::Const(val) => write!(f, "{}", val),
            Param::Param(param) => match &param.def {
                ParamDef::Constant(val) => write!(f, "Constant({})", val),
                ParamDef::RandFlat(min, max) => write!(f, "RandFlat(min={}, max={})", min, max),
                ParamDef::RandNorm(mean, stdev) => write!(f, "RandNorm(mean={}, stdev={})", mean, stdev),
                ParamDef::Changing(start, velocity) => write!(f, "Changing(start={}, velocity={})", start, velocity),
                ParamDef::Wave(shape, min, max, duration) => write!(f, "Wave(shape={:?}, min={}, max={}, duration={})", shape, min, max, duration),
                ParamDef::WaveCycle(shape, min, max, period, offset) => write!(f, "WaveCycle(shape={:?}, min={}, max={}, period={}, offset={})", shape, min, max, period, offset),
                ParamDef::Quote(param) => write!(f, "Quote({:?})", *param)
            },
        }
    }
}

impl Param {
    pub fn new(pdef: ParamDef) -> Param {
        let eparam = EParam {
            def: pdef,
            args: Vec::default(),
        };
        Param::Param(Box::new(eparam))
    }
    
    pub fn newconst(val: f32) -> Param {
        Param::Const(val)
    }
    
    pub fn eval(&self, ctx: &RunContext, age: f32) -> f32 {
        match self {
            Param::Const(val) => *val,
            Param::Param(param) => match &param.def {
                ParamDef::Constant(val) => *val,
                ParamDef::RandFlat(min, max) => {
                    let mut rng = ctx.rng.borrow_mut();
                    rng.gen_range(*min..*max)
                },
                ParamDef::RandNorm(mean, stdev) => {
                    let mut rng = ctx.rng.borrow_mut();
                    let val = rng.gen_range(0.0..1.0) + rng.gen_range(0.0..1.0) + rng.gen_range(0.0..1.0) - 1.5;
                    (val * stdev / 0.522) + mean
                },
                ParamDef::Changing(start, velocity) => {
                    start + age * velocity
                },
                ParamDef::Wave(shape, min, max, dur) => {
                    shape.sample(age/dur) * (max-min) + min
                },
                ParamDef::WaveCycle(shape, min, max, period, offset) => {
                    shape.sample(((age-offset)/period).rem_euclid(1.0)) * (max-min) + min
                },
                ParamDef::Quote(_) => {
                    panic!("eval Quote");
                },
            }
        }
    }

    pub fn min(&self, _ctx: &RunContext, age: f32) -> Option<f32> {
        match self {
            Param::Const(val) => Some(*val),
            Param::Param(param) => match &param.def {
                ParamDef::Constant(val) => Some(*val),
                ParamDef::RandFlat(min, _max) => Some(*min),
                ParamDef::RandNorm(mean, stdev) => {
                    Some((-1.5 * stdev / 0.522) + mean)
                },
                ParamDef::Changing(start, velocity) => {
                    if *velocity < 0.0 {
                        None
                    }
                    else {
                        Some(start + age * velocity)
                    }
                },
                ParamDef::Wave(_shape, min, _max, _period) => Some(*min),
                ParamDef::WaveCycle(_shape, min, _max, _period, _offset) => Some(*min),
                ParamDef::Quote(_) => {
                    panic!("eval Quote");
                },
            },
        }
    }

    pub fn max(&self, _ctx: &RunContext, age: f32) -> Option<f32> {
        match self {
            Param::Const(val) => Some(*val),
            Param::Param(param) => match &param.def {
                ParamDef::Constant(val) => Some(*val),
                ParamDef::RandFlat(_min, max) => Some(*max),
                ParamDef::RandNorm(mean, stdev) => {
                    Some((1.5 * stdev / 0.522) + mean)
                },
                ParamDef::Changing(start, velocity) => {
                    if *velocity > 0.0 {
                        None
                    }
                    else {
                        Some(start + age * velocity)
                    }
                },
                ParamDef::Wave(_shape, _min, max, _period) => Some(*max),
                ParamDef::WaveCycle(_shape, _min, max, _period, _offset) => Some(*max),
                ParamDef::Quote(_) => {
                    panic!("eval Quote");
                },
            },
        }
    }

    pub fn resolve(&self, ctx: &RunContext, age: f32) -> Param {
        match self {
            Param::Const(_) => self.clone(),
            Param::Param(param) => match &param.def {
                ParamDef::Quote(param) => *param.clone(),
                _ => Param::newconst(self.eval(ctx, age)),
            },
        }
    }

}

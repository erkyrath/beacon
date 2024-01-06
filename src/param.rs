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
    Constant(f32), //### lose this maybe
    RandFlat(usize, usize),  // min, max
    RandNorm(usize, usize),  // mean, stddev
    Changing(usize, usize),  // start, velocity
    Wave(WaveShape, usize, usize, usize), // shape, min, max, duration
    WaveCycle(WaveShape, usize, usize, usize, usize), // shape, min, max, period, offset

    Quote(usize),   // quotedparam
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
                ParamDef::RandFlat(min, max) => write!(f, "RandFlat(min={:?}, max={:?})", param.args[*min], param.args[*max]),
                ParamDef::RandNorm(mean, stdev) => write!(f, "RandNorm(mean={:?}, stdev={:?})", param.args[*mean], param.args[*stdev]),
                ParamDef::Changing(start, velocity) => write!(f, "Changing(start={:?}, velocity={:?})", param.args[*start], param.args[*velocity]),
                ParamDef::Wave(shape, min, max, duration) => write!(f, "Wave(shape={:?}, min={:?}, max={:?}, duration={:?})", shape, param.args[*min], param.args[*max], param.args[*duration]),
                ParamDef::WaveCycle(shape, min, max, period, offset) => write!(f, "WaveCycle(shape={:?}, min={:?}, max={:?}, period={:?}, offset={:?})", shape, param.args[*min], param.args[*max], param.args[*period], param.args[*offset]),
                ParamDef::Quote(subp) => write!(f, "Quote({:?})", param.args[*subp])
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

    pub fn addchild(mut self, child: Param) -> Param {
        if let Param::Param(ref mut param) = self {
            param.args.push(child);
        }
        else {
            panic!("cannot addchild to a Const");
        }
        self
    }
    
    pub fn eval(&self, ctx: &RunContext, age: f32) -> f32 {
        match self {
            Param::Const(val) => *val,
            Param::Param(param) => match &param.def {
                ParamDef::Constant(val) => *val,
                ParamDef::RandFlat(min, max) => {
                    let mut rng = ctx.rng.borrow_mut();
                    let min = param.args[*min].eval(ctx, age);
                    let max = param.args[*max].eval(ctx, age);
                    rng.gen_range(min..max)
                },
                ParamDef::RandNorm(mean, stdev) => {
                    let mut rng = ctx.rng.borrow_mut();
                    let mean = param.args[*mean].eval(ctx, age);
                    let stdev = param.args[*stdev].eval(ctx, age);
                    let val = rng.gen_range(0.0..1.0) + rng.gen_range(0.0..1.0) + rng.gen_range(0.0..1.0) - 1.5;
                    (val * stdev / 0.522) + mean
                },
                ParamDef::Changing(start, velocity) => {
                    let start = param.args[*start].eval(ctx, age);
                    let velocity = param.args[*velocity].eval(ctx, age);
                    start + age * velocity
                },
                ParamDef::Wave(shape, min, max, dur) => {
                    let min = param.args[*min].eval(ctx, age);
                    let max = param.args[*max].eval(ctx, age);
                    let dur = param.args[*dur].eval(ctx, age);
                    shape.sample(age/dur) * (max-min) + min
                },
                ParamDef::WaveCycle(shape, min, max, period, offset) => {
                    let min = param.args[*min].eval(ctx, age);
                    let max = param.args[*max].eval(ctx, age);
                    let period = param.args[*period].eval(ctx, age);
                    let offset = param.args[*offset].eval(ctx, age);
                    shape.sample(((age-offset)/period).rem_euclid(1.0)) * (max-min) + min
                },
                ParamDef::Quote(_) => {
                    panic!("eval Quote");
                },
            }
        }
    }

    pub fn min(&self, ctx: &RunContext, age: f32) -> Option<f32> {
        match self {
            Param::Const(val) => Some(*val),
            Param::Param(param) => match &param.def {
                ParamDef::Constant(val) => Some(*val),
                ParamDef::RandFlat(min, _max) => {
                    let min = param.args[*min].min(ctx, age);
                    min
                },
                ParamDef::RandNorm(mean, stdev) => {
                    let mean = param.args[*mean].min(ctx, age)?;
                    let stdev = param.args[*stdev].max(ctx, age)?;
                    Some((-1.5 * stdev / 0.522) + mean)
                },
                ParamDef::Changing(start, velocity) => {
                    let start = param.args[*start].min(ctx, age)?;
                    let velocity = param.args[*velocity].min(ctx, age)?;
                    if velocity < 0.0 {
                        None
                    }
                    else {
                        Some(start + age * velocity)
                    }
                },
                ParamDef::Wave(_shape, min, _max, _period) => {
                    let min = param.args[*min].min(ctx, age);
                    min
                },
                ParamDef::WaveCycle(_shape, min, _max, _period, _offset) => {
                    let min = param.args[*min].min(ctx, age);
                    min
                }
                ParamDef::Quote(_) => {
                    panic!("eval Quote");
                },
            },
        }
    }

    pub fn max(&self, ctx: &RunContext, age: f32) -> Option<f32> {
        match self {
            Param::Const(val) => Some(*val),
            Param::Param(param) => match &param.def {
                ParamDef::Constant(val) => Some(*val),
                ParamDef::RandFlat(_min, max) => {
                    let max = param.args[*max].max(ctx, age);
                    max
                },
                ParamDef::RandNorm(mean, stdev) => {
                    let mean = param.args[*mean].max(ctx, age)?;
                    let stdev = param.args[*stdev].max(ctx, age)?;
                    Some((1.5 * stdev / 0.522) + mean)
                },
                ParamDef::Changing(start, velocity) => {
                    let start = param.args[*start].max(ctx, age)?;
                    let velocity = param.args[*velocity].max(ctx, age)?;
                    if velocity > 0.0 {
                        None
                    }
                    else {
                        Some(start + age * velocity)
                    }
                },
                ParamDef::Wave(_shape, _min, max, _period) => {
                    let max = param.args[*max].max(ctx, age);
                    max
                },
                ParamDef::WaveCycle(_shape, _min, max, _period, _offset) => {
                    let max = param.args[*max].max(ctx, age);
                    max
                },
                ParamDef::Quote(_) => {
                    panic!("eval Quote");
                },
            },
        }
    }

    pub fn resolve(&self, ctx: &RunContext, age: f32) -> Param {
        match self {
            Param::Const(val) => Param::newconst(*val),
            Param::Param(param) => match &param.def {
                ParamDef::Quote(subp) => {
                    param.args[*subp].clone()
                },
                _ => Param::newconst(self.eval(ctx, age)),
            },
        }
    }

}

use std::fmt;
use std::cell::RefCell;

use crate::context::RunContext;
use crate::pixel::Pix;
use crate::waves::WaveShape;
use crate::param::Param;
use crate::pulser::{Pulser, PulserState};
use crate::script::ScriptIndex;

#[derive(Clone)]
pub enum Op1Def {
    Constant(f32),
    Wave(WaveShape, Param, Param, Param, Param), // wave, min, max, pos, width
    WaveCycle(WaveShape, Param, Param, Param, Param), // wave, min, max, pos, period
    Invert(), // op1
    Pulser(Pulser),
    Brightness(), // op3
    Mul(), // op1, op1
    Sum(), // op1...
    Clamp(Param, Param), // min, max; op1
}

#[derive(Clone)]
pub enum Op3Def {
    Constant(Pix<f32>),
    Invert(), // op3
    Grey(), // op1
    RGB(), // op1, op1, op1
    Gradient(Vec<Pix<f32>>), // stops; op1
    MulS(), // op3, op1
    Sum(), // op3...
}

impl Op1Def {
    pub fn describe(&self, indent: Option<String>) -> String {
        match self {
            Op1Def::Constant(val) => {
                format!("Constant({})", val)
            },
            Op1Def::Wave(shape, min, max, pos, width) => {
                format!("Wave({:?}, min={:?}, max={:?}, pos={:?}, width={:?})", shape, min, max, pos, width)
            },
            Op1Def::WaveCycle(shape, min, max, pos, period) => {
                format!("Wave({:?}, min={:?}, max={:?}, pos={:?}, period={:?})", shape, min, max, pos, period)
            },
            Op1Def::Invert() => {
                format!("Invert()")
            },
            Op1Def::Pulser(pulser) => {
                let limitstr = if let Some(size) = pulser.countlimit {
                    format!(", countlimit={}", size)
                } else {
                    String::default()
                };
                let indentstr = if let Some(val) = indent {
                    val
                } else {
                    " ".to_string()
                };
                let desc = format!(
                    "Pulser(interval={:?}{},{}duration={:?},{}pos={:?},{}width={:?},{}spaceshape={:?}, timeshape={:?})",
                    pulser.interval, limitstr,
                    indentstr, pulser.duration,
                    indentstr, pulser.pos,
                    indentstr, pulser.width,
                    indentstr, pulser.spaceshape, pulser.timeshape);
                desc
            },
            Op1Def::Brightness() => {
                format!("Brightness()")
            },
            Op1Def::Mul() => {
                format!("Mul()")
            },
            Op1Def::Sum() => {
                format!("Sum()")
            },
            Op1Def::Clamp(min, max) => {
                format!("Clamp({:?}, {:?})", min, max)
            },
            //_ => "?Op1Def".to_string(),
        }
    }
}

impl Op3Def {
    pub fn describe(&self, _indent: Option<String>) -> String {
        match self {
            Op3Def::Constant(pix) => {
                format!("Constant(r={}, g={}, b={})", pix.r, pix.g, pix.b)
            },
            Op3Def::Invert() => {
                format!("Invert()")
            },
            Op3Def::Grey() => {
                format!("Grey()")
            },
            Op3Def::RGB() => {
                format!("RGB()")
            },
            Op3Def::Gradient(stops) => {
                format!("Gradient({:?})", stops)
            },
            Op3Def::MulS() => {
                format!("MulS()")
            },
            Op3Def::Sum() => {
                format!("Sum()")
            },
            //_ => "?Op1Def".to_string(),
        }
    }
}

impl fmt::Debug for Op1Def {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.describe(None))
    }
}

impl fmt::Debug for Op3Def {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.describe(None))
    }
}

pub enum Op1State {
    NoState,
    Pulser(PulserState),
}

pub enum Op3State {
    NoState,
}

pub struct Op1Ctx {
    pub state: RefCell<Op1State>,
    pub buf: RefCell<Vec<f32>>,
}

pub struct Op3Ctx {
    pub state: RefCell<Op3State>,
    pub buf: RefCell<Vec<Pix<f32>>>,
}

impl Op1State {
    pub fn new_for(op: &Op1Def) -> Op1State {
        match op {
            Op1Def::Pulser(_pulser) => Op1State::Pulser(PulserState::new()),
            _ => Op1State::NoState,
        }
    }
}

impl Op3State {
    pub fn new_for(op: &Op3Def) -> Op3State {
        match op {
            _ => Op3State::NoState,
        }
    }
}

impl Op1Ctx {
    pub fn tickop(ctx: &mut RunContext, bufnum: usize) {
        let opref = &ctx.script.op1s[bufnum];
        let mut buf = ctx.op1s[bufnum].buf.borrow_mut();
        match &opref.op {
            Op1Def::Constant(val) => {
                for ix in 0..buf.len() {
                    buf[ix] = *val;
                }
            }

            Op1Def::Wave(shape, min, max, pos, width) => {
                let age = ctx.age() as f32;
                let width = width.eval(ctx, age);
                let startpos = pos.eval(ctx, age) - width*0.5;
                let min = min.eval(ctx, age);
                let max = max.eval(ctx, age);
                let buflen32 = buf.len() as f32;
                for ix in 0..buf.len() {
                    let basepos = ix as f32 / buflen32;
                    buf[ix] = shape.sample((basepos-startpos) / width) * (max-min) + min;
                }
            }

            Op1Def::WaveCycle(shape, min, max, pos, period) => {
                let age = ctx.age() as f32;
                let period = period.eval(ctx, age);
                let startpos = pos.eval(ctx, age) - period*0.5;
                let min = min.eval(ctx, age);
                let max = max.eval(ctx, age);
                let buflen32 = buf.len() as f32;
                for ix in 0..buf.len() {
                    let basepos = ix as f32 / buflen32;
                    buf[ix] = shape.sample(((basepos-startpos) / period).rem_euclid(1.0)) * (max-min) + min;
                }
            }

            Op1Def::Invert() => {
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    buf[ix] = 1.0 - obuf[ix];
                }
            }

            Op1Def::Pulser(pulser) => {
                let mut state = ctx.op1s[bufnum].state.borrow_mut();
                if let Op1State::Pulser(pstate) = &mut *state {
                    pstate.tick(ctx, &pulser);
                    pstate.render(ctx, &mut buf);
                }
                else {
                    panic!("Op1 state mismatch: PulserState");
                }
            }

            Op1Def::Mul() => {
                let obufnum1 = opref.get_type_ref(1, 0);
                let obufnum2 = opref.get_type_ref(1, 1);
                let obuf1 = ctx.op1s[obufnum1].buf.borrow();
                let obuf2 = ctx.op1s[obufnum2].buf.borrow();
                assert!(buf.len() == obuf1.len());
                assert!(buf.len() == obuf2.len());
                for ix in 0..buf.len() {
                    buf[ix] = obuf1[ix]*obuf2[ix]
                }
            }
            
            Op1Def::Sum() => {
                if opref.bufs.len() == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = 0.0;
                    }
                }
                else {
                    let obufnum = opref.get_type_ref(1, 0);
                    let obuf1 = ctx.op1s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix];
                    }
                    for jx in 1..opref.bufs.len() {
                        let obufnum = opref.get_type_ref(1, jx);
                        let obuf = ctx.op1s[obufnum].buf.borrow();
                        for ix in 0..buf.len() {
                            buf[ix] += obuf[ix];
                        }
                    }
                }
            }
            
            Op1Def::Clamp(min, max) => {
                let age = ctx.age() as f32;
                let min = min.eval(ctx, age);
                let max = max.eval(ctx, age);
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    buf[ix] = obuf[ix].clamp(min, max);
                }
            }

            _ => {
                panic!("unimplemented Op1");
            }
        }
    }
}

impl Op3Ctx {
    pub fn tickop(ctx: &mut RunContext, bufnum: usize) {
        let opref = &ctx.script.op3s[bufnum];
        //let mut _state = ctx.op3s[bufnum].state.borrow_mut();
        let mut buf = ctx.op3s[bufnum].buf.borrow_mut();
        match &opref.op {
            Op3Def::Constant(val) => {
                for ix in 0..buf.len() {
                    buf[ix] = val.clone();
                }
            }
            
            Op3Def::Invert() => {
                let obufnum = opref.get_type_ref(3, 0);
                let obuf = ctx.op3s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    buf[ix] = Pix::new(1.0-obuf[ix].r, 1.0-obuf[ix].g, 1.0-obuf[ix].b);
                }
            }

            Op3Def::RGB() => {
                let obufnum1 = opref.get_type_ref(1, 0);
                let obufnum2 = opref.get_type_ref(1, 1);
                let obufnum3 = opref.get_type_ref(1, 2);
                let obuf1 = ctx.op1s[obufnum1].buf.borrow();
                let obuf2 = ctx.op1s[obufnum2].buf.borrow();
                let obuf3 = ctx.op1s[obufnum3].buf.borrow();
                assert!(buf.len() == obuf1.len());
                assert!(buf.len() == obuf2.len());
                assert!(buf.len() == obuf3.len());
                for ix in 0..buf.len() {
                    buf[ix] = Pix::new(obuf1[ix], obuf2[ix], obuf3[ix]);
                }
            }

            Op3Def::Grey() => {
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    buf[ix] = Pix::new(obuf[ix], obuf[ix], obuf[ix]);
                }
            }

            Op3Def::Gradient(stops) => {
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    //###
                    buf[ix] = Pix::new(obuf[ix], obuf[ix], obuf[ix]);
                }
            }

            Op3Def::MulS() => {
                let obufnum1 = opref.get_type_ref(3, 0);
                let obufnum2 = opref.get_type_ref(1, 1);
                let obuf1 = ctx.op3s[obufnum1].buf.borrow();
                let obuf2 = ctx.op1s[obufnum2].buf.borrow();
                assert!(buf.len() == obuf1.len());
                assert!(buf.len() == obuf2.len());
                for ix in 0..buf.len() {
                    buf[ix] = Pix::new(obuf1[ix].r*obuf2[ix], obuf1[ix].g*obuf2[ix], obuf1[ix].b*obuf2[ix]);
                }
            }

            Op3Def::Sum() => {
                if opref.bufs.len() == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = Pix::new(0.0, 0.0, 0.0);
                    }
                }
                else {
                    let obufnum = opref.get_type_ref(3, 0);
                    let obuf1 = ctx.op3s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix].clone();
                    }
                    for jx in 1..opref.bufs.len() {
                        let obufnum = opref.get_type_ref(3, jx);
                        let obuf = ctx.op3s[obufnum].buf.borrow();
                        for ix in 0..buf.len() {
                            buf[ix].r += obuf[ix].r;
                            buf[ix].g += obuf[ix].g;
                            buf[ix].b += obuf[ix].b;
                        }
                    }
                }
            }
            
            //_ => { panic!("unimplemented Op3"); }
        }
    }
}

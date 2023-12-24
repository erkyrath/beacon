use std::fmt;
use std::cell::RefCell;

use crate::context::RunContext;
use crate::pixel::Pix;
use crate::pulser::{Pulser, PulserState};
use crate::script::ScriptIndex;

pub enum Op1Def {
    Constant(f32),
    Invert(usize), // op1
    Pulser(Pulser),
    Brightness(usize), // op3
    Sum(Vec<usize>), // op1...
}

pub enum Op3Def {
    Constant(Pix<f32>),
    Invert(usize), // op3
    Grey(usize), // op1
    RGB(usize, usize, usize), // op1, op1, op1
    CMulS(usize, usize), // op3, op1
    Sum(Vec<usize>), // op3...
}

impl Op1Def {
    pub fn describe(&self) -> (String, Vec<ScriptIndex>) {
        match self {
            Op1Def::Constant(val) => {
                (format!("Constant({})", val), Vec::default())
            },
            Op1Def::Invert(bufnum) => {
                (format!("Invert(1/{bufnum})"), vec![ ScriptIndex::Op1(*bufnum) ])
            },
            Op1Def::Pulser(pulser) => {
                let limitstr = if let Some(size) = pulser.countlimit {
                    format!(", countlimit={}", size)
                } else {
                    String::default()
                };
                (format!("Pulser(interval={:?}{}, duration={:?}, pos={:?}, width={:?}, spaceshape={:?}, timeshape={:?})", pulser.interval, limitstr, pulser.duration, pulser.pos, pulser.width, pulser.spaceshape, pulser.timeshape), Vec::default())
            },
            Op1Def::Brightness(bufnum) => {
                (format!("Brightness(3/{bufnum})"), vec![ ScriptIndex::Op3(*bufnum) ])
            },
            Op1Def::Sum(bufnums) => {
                let str = bufnums.iter().map(|val| format!("1/{}", val)).collect::<Vec<String>>().join(",");
                let ls = bufnums.iter().map(|val| ScriptIndex::Op1(*val)).collect::<Vec<ScriptIndex>>();
                (format!("Sum({str})"), ls)
            },
            //_ => ("?Op1Def".to_string(), Vec::default()),
        }
    }
}

impl Op3Def {
    pub fn describe(&self) -> (String, Vec<ScriptIndex>) {
        match self {
            Op3Def::Constant(pix) => {
                (format!("Constant(r={}, g={}, b={})", pix.r, pix.g, pix.b), Vec::default())
            },
            Op3Def::Invert(bufnum) => {
                (format!("Invert(3/{bufnum})"), vec![ ScriptIndex::Op3(*bufnum) ])
            },
            Op3Def::Grey(bufnum) => {
                (format!("Grey(1/{bufnum})"), vec![ ScriptIndex::Op1(*bufnum) ])
            },
            Op3Def::RGB(bufnum1, bufnum2, bufnum3) => {
                (format!("RGB(1/{bufnum1}, 1/{bufnum2}, 1/{bufnum3})"), vec![ ScriptIndex::Op1(*bufnum1), ScriptIndex::Op1(*bufnum2), ScriptIndex::Op1(*bufnum3) ])
            },
            Op3Def::CMulS(bufnum1, bufnum2) => {
                (format!("CMulS(3/{bufnum1}, 1/{bufnum2})"), vec![ ScriptIndex::Op3(*bufnum1), ScriptIndex::Op1(*bufnum2) ])
            },
            Op3Def::Sum(bufnums) => {
                let str = bufnums.iter().map(|val| format!("3/{}", val)).collect::<Vec<String>>().join(",");
                let ls = bufnums.iter().map(|val| ScriptIndex::Op3(*val)).collect::<Vec<ScriptIndex>>();
                (format!("Sum({str})"), ls)
            },
            //_ => ("?Op1Def".to_string(), Vec::default()),
        }
    }
}

impl fmt::Debug for Op1Def {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (desc, _bufs) = self.describe();
        write!(f, "{}", desc)
    }
}

impl fmt::Debug for Op3Def {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (desc, _bufs) = self.describe();
        write!(f, "{}", desc)
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
        let opdef = &ctx.script.op1s[bufnum];
        let mut buf = ctx.op1s[bufnum].buf.borrow_mut();
        match &opdef {
            Op1Def::Constant(val) => {
                for ix in 0..buf.len() {
                    buf[ix] = *val;
                }
            }

            Op1Def::Invert(obufnum) => {
                let obuf = ctx.op1s[*obufnum].buf.borrow();
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
            
            _ => {
                panic!("unimplemented Op1");
            }
        }
    }
}

impl Op3Ctx {
    pub fn tickop(ctx: &mut RunContext, bufnum: usize) {
        let opdef = &ctx.script.op3s[bufnum];
        //let mut _state = ctx.op3s[bufnum].state.borrow_mut();
        let mut buf = ctx.op3s[bufnum].buf.borrow_mut();
        match &opdef {
            Op3Def::Constant(val) => {
                for ix in 0..buf.len() {
                    buf[ix] = val.clone();
                }
            }
            
            Op3Def::Invert(obufnum) => {
                let obuf = ctx.op3s[*obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    buf[ix] = Pix::new(1.0-obuf[ix].r, 1.0-obuf[ix].g, 1.0-obuf[ix].b);
                }
            }

            Op3Def::RGB(obufnum1, obufnum2, obufnum3) => {
                let obuf1 = ctx.op1s[*obufnum1].buf.borrow();
                let obuf2 = ctx.op1s[*obufnum2].buf.borrow();
                let obuf3 = ctx.op1s[*obufnum3].buf.borrow();
                assert!(buf.len() == obuf1.len());
                assert!(buf.len() == obuf2.len());
                assert!(buf.len() == obuf3.len());
                for ix in 0..buf.len() {
                    buf[ix] = Pix::new(obuf1[ix], obuf2[ix], obuf3[ix]);
                }
            }

            Op3Def::CMulS(obufnum1, obufnum2) => {
                let obuf1 = ctx.op3s[*obufnum1].buf.borrow();
                let obuf2 = ctx.op1s[*obufnum2].buf.borrow();
                assert!(buf.len() == obuf1.len());
                assert!(buf.len() == obuf2.len());
                for ix in 0..buf.len() {
                    buf[ix] = Pix::new(obuf1[ix].r*obuf2[ix], obuf1[ix].g*obuf2[ix], obuf1[ix].b*obuf2[ix]);
                }
            }

            _ => {
                panic!("unimplemented Op3");
            }
        }
    }
}

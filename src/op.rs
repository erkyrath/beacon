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
    Grey(usize), // op1
    RGB(usize, usize, usize), // op1, op1, op1
    Sum(Vec<usize>), // op3...
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

pub fn tickop(ctx: &mut RunContext, scix: ScriptIndex) {
    match scix {
        ScriptIndex::Op1(val) => {
            let opdef = &ctx.script.op1s[val];
            let mut buf = ctx.op1s[val].buf.borrow_mut();
            match &opdef {
                Op1Def::Constant(val) => {
                    for ix in 0..buf.len() {
                        buf[ix] = *val;
                    }
                }

                Op1Def::Pulser(_pulser) => {
                    let mut state = ctx.op1s[val].state.borrow_mut();
                    if let Op1State::Pulser(pstate) = &mut *state {
                        pstate.tick(ctx);
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
        },
        ScriptIndex::Op3(val) => {
            let opdef = &ctx.script.op3s[val];
            //let mut _state = ctx.op3s[val].state.borrow_mut();
            let mut buf = ctx.op3s[val].buf.borrow_mut();
            match &opdef {
                Op3Def::Constant(val) => {
                    for ix in 0..buf.len() {
                        buf[ix] = val.clone();
                    }
                }
                
                _ => {
                    panic!("unimplemented Op3");
                }
            }
        },
    }
}


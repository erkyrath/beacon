use std::cell::RefCell;

use crate::context::RunContext;
use crate::pixel::Pix;
use crate::pulser::{Pulser, PulserState};

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

/*###
impl Op1Ctx {
    pub fn tick(&mut self, _ctx: &mut RunContext, opdef: &Op1Def) {
        match &opdef {
            Op1Def::Constant(val) => {
                for ix in 0..self.buf.len() {
                    self.buf[ix] = *val;
                }
            }

            Op1Def::Pulser(_pulser) => {
                if let Op1State::Pulser(state) = self.state {
                    state.tick(&ctx);
                    state.render(&ctx, &mut self.buf);
                }
                else {
                    panic!("Op1 state mismatch: PulserState");
                }
            }

            Op1Def::Invert(_src) => {
                for ix in 0..self.buf.len() {
                    self.buf[ix] = 0.5; //### script.op1s[src].buf
                }
            }

            _ => {
                panic!("unimplemented Op1");
            }
        }
    }
}

impl Op3Ctx {
    pub fn tick(&mut self, _ctx: &mut RunContext, opdef: &Op3Def) {
        match &opdef {
            Op3Def::Constant(val) => {
                for ix in 0..self.buf.len() {
                    self.buf[ix] = val.clone();
                }
            }

            _ => {
                panic!("unimplemented Op3");
            }
        }
    }
}

###*/

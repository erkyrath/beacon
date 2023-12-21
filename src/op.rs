use crate::context::RunContext;
use crate::pixel::Pix;
use crate::pulser::Pulser;

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

pub struct Op1 {
    pub def: Op1Def,
    pub buf: Vec<f32>,
}

pub struct Op3 {
    pub def: Op3Def,
    pub buf: Vec<Pix<f32>>,
}

impl Op1 {
    pub fn tick(&mut self, ctx: &RunContext) {
        match &mut self.def {
            Op1Def::Constant(val) => {
                for ix in 0..self.buf.len() {
                    self.buf[ix] = *val;
                }
            }

            Op1Def::Pulser(pulser) => {
                pulser.tick(&ctx);
                pulser.render(&ctx, &mut self.buf);
            }

            Op1Def::Invert(src) => {
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

impl Op3 {
    pub fn tick(&mut self, _ctx: &RunContext) {
        match &mut self.def {
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


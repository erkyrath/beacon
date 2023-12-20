
use crate::pixel::Pix;
use crate::pulser::Pulser;

pub enum Op1Def {
    Constant(f32),
    Pulser(Pulser),
    Brightness(u32), // op3
    Sum(Vec<u32>), // op1...
}

pub enum Op3Def {
    Constant(Pix<f32>),
    Grey(u32), // op1
    RGB(u32, u32, u32), // op1, op1, op1
    Sum(Vec<u32>), // op3...
}

pub struct Op1 {
    def: Op1Def,
    buf: Vec<f32>,
}

pub struct Op3 {
    def: Op3Def,
    buf: Vec<Pix<f32>>,
}

pub enum ScriptIndex {
    Op1(u32),
    Op3(u32),
}

pub struct Script {
    order: Vec<ScriptIndex>, // 0 is root
    op1s: Vec<Op1>,
    op3s: Vec<Op3>,
}


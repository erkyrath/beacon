
use crate::pixel::Pix;
use crate::pulser::Pulser;

pub enum Op1Def {
    Constant(f32),
    Pulser(Pulser),
}

pub enum Op3Def {
    Constant(Pix<f32>),
}

pub struct Op1 {
    def: Op1Def,
    buf: Vec<f32>,
}

pub struct Op3 {
    def: Op3Def,
    buf: Vec<Pix<f32>>,
}

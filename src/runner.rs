use crate::pixel::Pix;

use crate::context::ScriptContext;

pub enum PixBuffer<'a> {
    Buf1(&'a [f32]),
    Buf3(&'a [Pix<f32>]),
}

pub trait Runner {
    fn build(&self, size: usize, fixtick: Option<u32>) -> RunContextWrap;
}

pub trait RunContext {
    fn applybuf<F>(&self, func: F)
    where F: FnMut(PixBuffer);

    fn done(&self) -> bool;
}

pub enum RunContextWrap {
    Script(ScriptContext),
}

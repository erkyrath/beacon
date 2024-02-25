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

impl RunContextWrap {
    pub fn tick(&mut self) {
        match self {
            RunContextWrap::Script(ctx) => ctx.tick()
        }
    }
    
    pub fn age(&self) -> f64 {
        match self {
            RunContextWrap::Script(ctx) => ctx.age()
        }
    }
}

impl RunContext for RunContextWrap {
    fn applybuf<F>(&self, func: F)
    where F: FnMut(PixBuffer) {
        match self {
            RunContextWrap::Script(ctx) => ctx.applybuf(func)
        }
    }
    
    fn done(&self) -> bool {
        match self {
            RunContextWrap::Script(ctx) => ctx.done()
        }
    }
}

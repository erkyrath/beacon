use crate::pixel::Pix;

use crate::context::scriptcontext::{ScriptRunner, ScriptContext};

pub enum PixBuffer<'a> {
    Buf1(&'a [f32]),
    Buf3(&'a [Pix<f32>]),
}

pub trait RunContext {
    fn tick(&mut self);

    fn age(&self) -> f64;

    fn applybuf<F>(&self, func: F)
    where F: FnMut(PixBuffer);

    fn done(&self) -> bool;
}

pub enum Runner {
    Script(ScriptRunner),
}

impl Runner {
    pub fn build(&self, size: usize, fixtick: Option<u32>) -> RunContextWrap {
        match self {
            Runner::Script(runner) => {
                let ctx = ScriptContext::new(runner.script.clone(), size, fixtick);
                RunContextWrap::Script(ctx)
            }
        }
    }
}

pub enum RunContextWrap {
    Script(ScriptContext),
}

impl RunContext for RunContextWrap {
    fn tick(&mut self) {
        match self {
            RunContextWrap::Script(ctx) => ctx.tick()
        }
    }
    
    fn age(&self) -> f64 {
        match self {
            RunContextWrap::Script(ctx) => ctx.age()
        }
    }

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

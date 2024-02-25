use crate::pixel::Pix;

use crate::context::scriptcontext::{ScriptRunner, ScriptContext};
use crate::context::limitcontext::{LimitRunner, LimitContext};
use crate::context::cyclecontext::{CycleRunner};

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

#[derive(Clone)]
pub enum Runner {
    Script(ScriptRunner),
    Limit(LimitRunner),
    Cycle(CycleRunner),
}

impl Runner {
    pub fn build(&self, size: usize, fixtick: Option<u32>) -> RunContextWrap {
        match self {
            Runner::Script(run) => {
                let ctx = ScriptContext::new(run.script.clone(), size, fixtick);
                RunContextWrap::Script(ctx)
            },
            Runner::Limit(run) => {
                let child = run.runner.build(size, fixtick);
                let ctx = LimitContext::new(child, run.limit, size, fixtick);
                RunContextWrap::Limit(ctx)
            },
            Runner::Cycle(run) => {
                panic!("###")
            },
        }
    }
}

pub enum RunContextWrap {
    Script(ScriptContext),
    Limit(LimitContext),
}

impl RunContext for RunContextWrap {
    fn tick(&mut self) {
        match self {
            RunContextWrap::Script(ctx) => ctx.tick(),
            RunContextWrap::Limit(ctx) => ctx.tick(),
        }
    }
    
    fn age(&self) -> f64 {
        match self {
            RunContextWrap::Script(ctx) => ctx.age(),
            RunContextWrap::Limit(ctx) => ctx.age(),
        }
    }

    fn applybuf<F>(&self, func: F)
    where F: FnMut(PixBuffer) {
        match self {
            RunContextWrap::Script(ctx) => ctx.applybuf(func),
            RunContextWrap::Limit(ctx) => ctx.applybuf(func),
        }
    }
    
    fn done(&self) -> bool {
        match self {
            RunContextWrap::Script(ctx) => ctx.done(),
            RunContextWrap::Limit(ctx) => ctx.done(),
        }
    }
}

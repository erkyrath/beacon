use crate::pixel::Pix;

use crate::context::scriptcontext::{ScriptRunner, ScriptContext};
use crate::context::limitcontext::{LimitRunner, LimitContext};
use crate::context::cyclecontext::{CycleRunner, CycleContext};
use crate::context::watchcontext::{WatchScriptRunner, WatchScriptContext};

pub enum PixBuffer<'a> {
    Buf1(&'a [f32]),
    Buf3(&'a [Pix<f32>]),
}

pub trait RunContext {
    fn tick(&mut self) -> Result<(), String>;

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
    WatchScript(WatchScriptRunner),
}

impl Runner {
    pub fn build(&self, size: usize, fixtick: Option<u32>) -> Result<RunContextWrap, String> {
        match self {
            Runner::Script(run) => run.build(size, fixtick),
            Runner::Limit(run) => run.build(size, fixtick),
            Runner::Cycle(run) => run.build(size, fixtick),
            Runner::WatchScript(run) => run.build(size, fixtick),
        }
    }

    pub fn getname(&self) -> &str {
        match self {
            Runner::Script(run) => run.getname(),
            Runner::Limit(run) => run.getname(),
            Runner::Cycle(run) => run.getname(),
            Runner::WatchScript(run) => run.getname(),
        }
    }
}

pub enum RunContextWrap {
    Script(ScriptContext),
    Limit(LimitContext),
    Cycle(CycleContext),
    WatchScript(WatchScriptContext),
}

impl RunContext for RunContextWrap {
    fn tick(&mut self) -> Result<(), String> {
        match self {
            RunContextWrap::Script(ctx) => ctx.tick(),
            RunContextWrap::Limit(ctx) => ctx.tick(),
            RunContextWrap::Cycle(ctx) => ctx.tick(),
            RunContextWrap::WatchScript(ctx) => ctx.tick(),
        }
    }
    
    fn age(&self) -> f64 {
        match self {
            RunContextWrap::Script(ctx) => ctx.age(),
            RunContextWrap::Limit(ctx) => ctx.age(),
            RunContextWrap::Cycle(ctx) => ctx.age(),
            RunContextWrap::WatchScript(ctx) => ctx.age(),
        }
    }

    fn applybuf<F>(&self, func: F)
    where F: FnMut(PixBuffer) {
        match self {
            RunContextWrap::Script(ctx) => ctx.applybuf(func),
            RunContextWrap::Limit(ctx) => ctx.applybuf(func),
            RunContextWrap::Cycle(ctx) => ctx.applybuf(func),
            RunContextWrap::WatchScript(ctx) => ctx.applybuf(func),
        }
    }
    
    fn done(&self) -> bool {
        match self {
            RunContextWrap::Script(ctx) => ctx.done(),
            RunContextWrap::Limit(ctx) => ctx.done(),
            RunContextWrap::Cycle(ctx) => ctx.done(),
            RunContextWrap::WatchScript(ctx) => ctx.done(),
        }
    }
}

impl RunContextWrap {
    pub fn applybufadd(&self, changebuf: &mut [Pix<f32>], scale: f32) {
        let pixsize = changebuf.len();
        
        self.applybuf(|pixbuf| {
            match pixbuf {
                PixBuffer::Buf1(buf) => {
                    assert!(pixsize == buf.len());
                    for xpos in 0..pixsize {
                        changebuf[xpos].r += scale * buf[xpos];
                        changebuf[xpos].g += scale * buf[xpos];
                        changebuf[xpos].b += scale * buf[xpos];
                    }
                },
                PixBuffer::Buf3(buf) => {
                    assert!(pixsize == buf.len());
                    for xpos in 0..pixsize {
                        changebuf[xpos].r += scale * buf[xpos].r;
                        changebuf[xpos].g += scale * buf[xpos].g;
                        changebuf[xpos].b += scale * buf[xpos].b;
                    }
                },
            }
        });
    }    
    
}

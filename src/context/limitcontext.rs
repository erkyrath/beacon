use crate::clock::CtxClock;
use crate::runner::{Runner, RunContext, RunContextWrap, PixBuffer};

pub struct LimitRunner {
    pub runner: Box<Runner>,
    pub limit: f32,
}

impl LimitRunner {
    pub fn new(runner: Runner, limit: f32) -> Runner {
        let run = LimitRunner {
            runner: Box::new(runner),
            limit: limit,
        };
        Runner::Limit(run)
    }
}

pub struct LimitContext {
    pub child: Box<RunContextWrap>,
    pub limit: f32,
    
    pub clock: CtxClock,
}

impl LimitContext {
    pub fn new(child: RunContextWrap, limit: f32, _size: usize, fixtick: Option<u32>) -> LimitContext {
        let ctx = LimitContext {
            child: Box::new(child),
            limit: limit,
            clock: CtxClock::new(fixtick),
        };
        ctx
    }
}

impl RunContext for LimitContext {

    fn tick(&mut self) {
        self.child.tick();
    }

    fn age(&self) -> f64 {
        self.clock.age
    }
    
    fn applybuf<F>(&self, func: F)
    where F: FnMut(PixBuffer) {
        self.child.applybuf(func);
    }

    fn done(&self) -> bool {
        self.clock.age as f32 > self.limit
    }
    
}

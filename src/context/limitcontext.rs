use crate::runner::{Runner, RunContext, RunContextWrap, PixBuffer};

#[derive(Clone)]
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

    pub fn getname(&self) -> &str {
        self.runner.getname()
    }
    
    pub fn build(&self, size: usize, fixtick: Option<u32>) -> Result<RunContextWrap, String> {
        let child = self.runner.build(size, fixtick)?;
        let ctx = LimitContext::new(child, self.limit, size, fixtick);
        Ok(RunContextWrap::Limit(ctx))
    }
}

pub struct LimitContext {
    child: Box<RunContextWrap>,
    limit: f32,
}

impl LimitContext {
    pub fn new(child: RunContextWrap, limit: f32, _size: usize, _fixtick: Option<u32>) -> LimitContext {
        let ctx = LimitContext {
            child: Box::new(child),
            limit: limit,
        };
        ctx
    }
}

impl RunContext for LimitContext {

    fn tick(&mut self) -> Result<(), String> {
        self.child.tick()
    }

    fn age(&self) -> f64 {
        self.child.age()
    }
    
    fn applybuf<F>(&self, func: F)
    where F: FnMut(PixBuffer) {
        self.child.applybuf(func);
    }

    fn done(&self) -> bool {
        if self.child.age() as f32 > self.limit {
            return true;
        }
        self.child.done()
    }
    
}

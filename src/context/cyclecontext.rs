use crate::runner::{Runner, RunContext, RunContextWrap, PixBuffer};
use crate::clock::CtxClock;

#[derive(Clone)]
pub struct CycleRunner {
    pub runners: Box<Vec<Runner>>,
    pub interval: f32,
}

impl CycleRunner {
    pub fn new(runners: Vec<Runner>, interval: f32) -> Runner {
        assert!(runners.len() > 0);
        let run = CycleRunner {
            runners: Box::new(runners),
            interval: interval,
        };
        Runner::Cycle(run)
    }
}

pub struct CycleContext {
    runners: Box<Vec<Runner>>,
    interval: f32,
    size: usize,
    clock: CtxClock,

    curindex: usize,
    curchild: Box<RunContextWrap>,
    nextchange: f32,
}

impl CycleContext {
    pub fn new(runners: Box<Vec<Runner>>, interval: f32, size: usize, fixtick: Option<u32>) -> CycleContext {
        let runner = runners[0].clone();
        let child = runner.build(size, fixtick);
        
        CycleContext {
            runners: runners,
            interval: interval,
            size: size,
            clock: CtxClock::new(fixtick),

            curindex: 0,
            curchild: Box::new(child),
            nextchange: interval,
        }
    }
}


impl RunContext for CycleContext {

    fn tick(&mut self) {
        let _newage: f64 = self.clock.tick();
        self.curchild.tick();
    }

    fn age(&self) -> f64 {
        self.clock.age
    }
    
    fn applybuf<F>(&self, func: F)
    where F: FnMut(PixBuffer) {
        self.curchild.applybuf(func);
    }

    fn done(&self) -> bool {
        self.curchild.done()
    }
    
}

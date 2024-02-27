use std::mem;
use std::cell::RefCell;

use crate::pixel::Pix;
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
    fadetime: f32,
    size: usize,
    fixtick: Option<u32>,
    clock: CtxClock,

    curindex: usize,
    curchild: Box<RunContextWrap>,
    lastchild: Option<Box<RunContextWrap>>,
    lastchange: f32,
    nextchange: f32,

    changebuf: RefCell<Vec<Pix<f32>>>,
}

impl CycleContext {
    pub fn new(runners: Box<Vec<Runner>>, interval: f32, size: usize, fixtick: Option<u32>) -> Result<CycleContext, String> {
        let runner = runners[0].clone();
        let child = runner.build(size, fixtick)?;
        
        let ctx = CycleContext {
            runners: runners,
            interval: interval,
            fadetime: 0.5,
            size: size,
            fixtick: fixtick,
            clock: CtxClock::new(fixtick),

            curindex: 0,
            curchild: Box::new(child),
            lastchild: None,
            lastchange: 0.0,
            nextchange: interval,

            changebuf: RefCell::new(vec![Pix::new(0.0, 0.0, 0.0); size]),
        };
        Ok(ctx)
    }
}

impl RunContext for CycleContext {

    fn tick(&mut self) {
        let newage = self.clock.tick() as f32;

        if newage > self.nextchange || self.curchild.done() {
            self.nextchange = newage + self.interval;
            self.curindex = (self.curindex+1) % self.runners.len();
            let runner = self.runners[self.curindex].clone();
            let newchild = runner.build(self.size, self.fixtick).unwrap(); //###
            let lastchild = mem::replace(&mut self.curchild, Box::new(newchild));
            self.lastchange = newage;
            self.lastchild = Some(lastchild);
        }
        
        if self.lastchild.is_some() && newage > self.lastchange + self.fadetime {
            self.lastchild = None;
        }
        
        self.curchild.tick();
        if let Some(child) = &mut self.lastchild {
            child.tick();
        }
    }

    fn age(&self) -> f64 {
        self.clock.age
    }
    
    fn applybuf<F>(&self, mut func: F)
    where F: FnMut(PixBuffer) {
        match &self.lastchild {
            None => self.curchild.applybuf(func),
            Some(child) => {
                {
                    let scale = (self.age() as f32 - self.lastchange) / self.fadetime;
                    let mut changebuf = self.changebuf.borrow_mut();
                    changebuf.fill(Pix::new(0.0, 0.0, 0.0));
                    self.curchild.applybufadd(&mut changebuf, scale);
                    child.applybufadd(&mut changebuf, 1.0-scale);
                }
                {
                    let changebuf = self.changebuf.borrow();
                    func(PixBuffer::Buf3(&changebuf));
                }
            }
        }
    }

    fn done(&self) -> bool {
        false
    }
    
}

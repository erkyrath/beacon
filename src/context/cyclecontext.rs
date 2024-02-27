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
    pub fn new(runners: Box<Vec<Runner>>, interval: f32, size: usize, fixtick: Option<u32>) -> CycleContext {
        let runner = runners[0].clone();
        let child = runner.build(size, fixtick);
        
        CycleContext {
            runners: runners,
            interval: interval,
            fadetime: 1.0,
            size: size,
            fixtick: fixtick,
            clock: CtxClock::new(fixtick),

            curindex: 0,
            curchild: Box::new(child),
            lastchild: None,
            lastchange: 0.0,
            nextchange: interval,

            changebuf: RefCell::new(vec![Pix::new(0.0, 0.0, 0.0); size]),
        }
    }
}


impl RunContext for CycleContext {

    fn tick(&mut self) {
        let newage = self.clock.tick() as f32;

        if newage > self.nextchange || self.curchild.done() {
            self.nextchange = newage + self.interval;
            self.curindex = (self.curindex+1) % self.runners.len();
            let runner = self.runners[self.curindex].clone();
            let lastchild = mem::replace(&mut self.curchild, Box::new(runner.build(self.size, self.fixtick)));
            self.lastchange = newage;
            self.lastchild = Some(lastchild);
        }
        
        if self.lastchild.is_some() && newage > self.lastchange + self.fadetime {
            self.lastchild = None;
        }
        
        self.curchild.tick();
    }

    fn age(&self) -> f64 {
        self.clock.age
    }
    
    fn applybuf<F>(&self, mut func: F)
    where F: FnMut(PixBuffer) {
        match &self.lastchild {
            None => self.curchild.applybuf(func),
            Some(child) => {
                let pixsize = self.size;
                /*###{
                    let mut changebuf = self.changebuf.borrow_mut();
                    changebuf.fill(Pix::new(0.0, 0.0, 0.0));
                    self.curchild.applybuf(|pixbuf| {
                        match pixbuf {
                            PixBuffer::Buf1(buf) => {
                                for xpos in 0..pixsize {
                                    changebuf[xpos].r += buf[xpos];
                                    changebuf[xpos].g += buf[xpos];
                                    changebuf[xpos].b += buf[xpos];
                                }
                            },
                            PixBuffer::Buf3(buf) => {
                                for xpos in 0..pixsize {
                                    changebuf[xpos].r += buf[xpos].r;
                                    changebuf[xpos].g += buf[xpos].r;
                                    changebuf[xpos].b += buf[xpos].r;
                                }
                            },
                        }
                    });
                    child.applybuf(|pixbuf| {
                        match pixbuf {
                            PixBuffer::Buf1(buf) => {
                                for xpos in 0..pixsize {
                                    changebuf[xpos].r += buf[xpos];
                                    changebuf[xpos].g += buf[xpos];
                                    changebuf[xpos].b += buf[xpos];
                                }
                            },
                            PixBuffer::Buf3(buf) => {
                                for xpos in 0..pixsize {
                                    changebuf[xpos].r += buf[xpos].r;
                                    changebuf[xpos].g += buf[xpos].r;
                                    changebuf[xpos].b += buf[xpos].r;
                                }
                            },
                        }
                    });
                }###*/
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

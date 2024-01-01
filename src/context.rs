use std::time::Instant;
use std::rc::Rc;
use std::cell::RefCell;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::pixel::Pix;
use crate::script::{Script, ScriptIndex};
use crate::op::{Op1Ctx, Op3Ctx};
use crate::op::{Op1Def, Op3Def};
use crate::op::{Op1State, Op3State};

pub struct RunContext {
    pub script: Script,
    pub size: usize,
    
    birth: Instant,
    age: f64,
    ticklen: f32,
    
    pub rng: Rc<RefCell<SmallRng>>,

    pub op1s: Vec<Op1Ctx>,
    pub op3s: Vec<Op3Ctx>,
}

impl RunContext {
    pub fn new(script: Script, size: usize) -> RunContext {
        // Gotta create this with some temporary values and then fill them in.
        let mut ctx = RunContext {
            script: Script::new(),
            size: size,
            birth: Instant::now(),
            age: 0.0,
            ticklen: 0.0,
            rng: Rc::new(RefCell::new(SmallRng::from_entropy())),
            op1s: Vec::default(),
            op3s: Vec::default(),
        };

        let mut op1s: Vec<Op1Ctx> = Vec::default();
        let mut op3s: Vec<Op3Ctx> = Vec::default();
        
        for op in &script.op1s {
            op1s.push(Op1Ctx {
                state: RefCell::new(Op1State::new_for(&op.op, &mut ctx)),
                buf: RefCell::new(vec![0.0; size]),
            });
        }
        
        for op in &script.op3s {
            op3s.push(Op3Ctx {
                state: RefCell::new(Op3State::new_for(&op.op, &mut ctx)),
                buf: RefCell::new(vec![Pix::new(0.0, 0.0, 0.0); size]),
            });
        }

        ctx.script = script;
        ctx.op1s = op1s;
        ctx.op3s = op3s;

        ctx
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn age(&self) -> f64 {
        self.age
    }
    
    pub fn ticklen(&self) -> f32 {
        self.ticklen
    }

    pub fn applybuf<F>(&self, mut func: F)
    where F: FnMut(Option<&[f32]>, Option<&[Pix<f32>]>) {
        match &self.script.order[0] {
            ScriptIndex::Op1(val) => {
                let buf = self.op1s[*val].buf.borrow();
                func(Some(&buf), None);
            },
            ScriptIndex::Op3(val) => {
                let buf = self.op3s[*val].buf.borrow();
                func(None, Some(&buf));
            },
        }
    }
    
    pub fn applybuf1<F>(&self, val: usize, mut func: F)
    where F: FnMut(&[f32]) {
        let buf = self.op1s[val].buf.borrow();
        func(&buf);
    }

    pub fn applybuf3<F>(&self, val: usize, mut func: F)
    where F: FnMut(&[Pix<f32>]) {
        let buf = self.op3s[val].buf.borrow();
        func(&buf);
    }

    pub fn tick(&mut self) {
        let dur = self.birth.elapsed();
        let newage = dur.as_secs_f64();
        self.ticklen = (newage - self.age) as f32;
        self.age = newage;

        for ix in (0..self.script.order.len()).rev() {
            match self.script.order[ix] {
                ScriptIndex::Op1(val) => {
                    Op1Ctx::tickop(self, val);
                },
                ScriptIndex::Op3(val) => {
                    Op3Ctx::tickop(self, val);
                },
            }
        }
    }
}

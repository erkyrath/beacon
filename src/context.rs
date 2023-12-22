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
    script: Script,
    size: usize,
    
    birth: Instant,
    age: f64,
    pub rng: Rc<RefCell<SmallRng>>,

    op1s: Vec<Op1Ctx>,
    op3s: Vec<Op3Ctx>,
}

pub enum ScriptBuffer<'a> {
    Op1(&'a [f32]),
    Op3(&'a [Pix<f32>]),
}

impl RunContext {
    pub fn new(script: Script, size: usize) -> RunContext {
        RunContext {
            script: script,
            size: size,
            birth: Instant::now(),
            age: 0.0,
            rng: Rc::new(RefCell::new(SmallRng::from_entropy())),
            op1s: Vec::default(), //###
            op3s: Vec::default(), //###
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn age(&self) -> f64 {
        self.age
    }
    
    pub fn getrootbuf(&self) -> ScriptBuffer {
        match &self.script.order[0] {
            ScriptIndex::Op1(val) => {
                ScriptBuffer::Op1(&self.op1s[*val].buf)
            },
            ScriptIndex::Op3(val) => {
                ScriptBuffer::Op3(&self.op3s[*val].buf)
            },
        }
    }

    pub fn tick(&mut self) {
        let dur = self.birth.elapsed();
        self.age = dur.as_secs_f64();
        
        for scix in (&self.script.order).iter().rev() {
            match scix {
                ScriptIndex::Op1(val) => {
                    //###self.op1s[*val].tick(self, &self.script.op1s[*val]);
                    let opdef = &self.script.op1s[*val];
                    let buf = &mut self.op1s[*val].buf;
                    let state = &mut self.op1s[*val].state;
                    match &opdef {
                        Op1Def::Constant(val) => {
                            for ix in 0..buf.len() {
                                buf[ix] = *val;
                            }
                        }
                        
                        Op1Def::Pulser(_pulser) => {
                            if let Op1State::Pulser(pstate) = state {
                                pstate.tick(self);
                                pstate.render(self, buf);
                            }
                            else {
                                panic!("Op1 state mismatch: PulserState");
                            }
                        }
                        
                        _ => {
                            panic!("unimplemented Op1");
                        }
                    }
                },
                ScriptIndex::Op3(val) => {
                    //###self.op3s[*val].tick(self, &self.script.op3s[*val]);
                    let opdef = &self.script.op3s[*val];
                    let buf = &mut self.op3s[*val].buf;
                    match &opdef {
                        Op3Def::Constant(val) => {
                            for ix in 0..buf.len() {
                                buf[ix] = val.clone();
                            }
                        }
                        
                        _ => {
                            panic!("unimplemented Op3");
                        }
                    }
                },
            }
        }
    }
}

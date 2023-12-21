use crate::context::RunContext;
use crate::pixel::Pix;
use crate::pulser::Pulser;

pub enum Op1Def {
    Constant(f32),
    Pulser(Pulser),
    Brightness(usize), // op3
    Sum(Vec<usize>), // op1...
}

pub enum Op3Def {
    Constant(Pix<f32>),
    Grey(usize), // op1
    RGB(usize, usize, usize), // op1, op1, op1
    Sum(Vec<usize>), // op3...
}

pub struct Op1 {
    def: Op1Def,
    buf: Vec<f32>,
}

pub struct Op3 {
    def: Op3Def,
    buf: Vec<Pix<f32>>,
}

impl Op1 {
    fn tick(&mut self, ctx: &RunContext) {
        match &self.def {
            Op1Def::Constant(val) => {
                for ix in 0..self.buf.len() {
                    self.buf[ix] = *val;
                }
            }

            _ => {
                panic!("unimplemented Op1");
            }
        }
    }
}

impl Op3 {
    fn tick(&mut self, ctx: &RunContext) {
        match &self.def {
            Op3Def::Constant(val) => {
                for ix in 0..self.buf.len() {
                    self.buf[ix] = val.clone();
                }
            }

            _ => {
                panic!("unimplemented Op3");
            }
        }
    }
}

pub enum ScriptIndex {
    Op1(usize),
    Op3(usize),
}

pub struct Script {
    order: Vec<ScriptIndex>, // 0 is root
    op1s: Vec<Op1>,
    op3s: Vec<Op3>,
}

impl Script {
    pub fn new() -> Script {
        Script {
            order: Vec::default(),
            op1s: Vec::default(),
            op3s: Vec::default(),
        }
    }

    pub fn tick(&mut self, ctx: &RunContext) {
        //### backwards please
        for scix in &self.order {
            match scix {
                ScriptIndex::Op1(val) => {
                    self.op1s[*val].tick(ctx);
                },
                ScriptIndex::Op3(val) => {
                    self.op3s[*val].tick(ctx);
                },
            }
        }
    }
}

pub fn build_script(ctx: &RunContext) -> Script {
    let mut script = Script::new();

    let pulser = Pulser::new(ctx);
    let op = Op1 {
        def: Op1Def::Pulser(pulser),
        buf: vec![0.0; ctx.size()],
    };

    script.op1s.push(op);
    script.order.push(ScriptIndex::Op1(0));
    
    return script;
}

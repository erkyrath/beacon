use crate::context::RunContext;
use crate::pixel::Pix;
use crate::op::{Op1, Op3, Op1Def, Op3Def};
use crate::pulser::Pulser;

pub enum ScriptIndex {
    Op1(usize),
    Op3(usize),
}

pub enum ScriptBuffer<'a> {
    Op1(&'a [f32]),
    Op3(&'a [Pix<f32>]),
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

    pub fn getrootbuf(&self) -> ScriptBuffer {
        match &self.order[0] {
            ScriptIndex::Op1(val) => {
                ScriptBuffer::Op1(&self.op1s[*val].buf)
            },
            ScriptIndex::Op3(val) => {
                ScriptBuffer::Op3(&self.op3s[*val].buf)
            },
        }
    }

    pub fn tick(&mut self, ctx: &RunContext) {
        for scix in (&self.order).iter().rev() {
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

    /*
    let inverter = Op1 {
        def: Op1Def::Invert(1),
        buf: vec![0.0; ctx.size()],
    };
    script.order.push(ScriptIndex::Op1(script.op1s.len()));
    script.op1s.push(inverter);
    */
    
    let pulser = Op1 {
        def: Op1Def::Pulser(Pulser::new()),
        buf: vec![0.0; ctx.size()],
    };
    script.order.push(ScriptIndex::Op1(script.op1s.len()));
    script.op1s.push(pulser);

    return script;
}

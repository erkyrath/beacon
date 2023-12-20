use crate::context::RunContext;
use crate::pixel::Pix;
use crate::pulser::Pulser;

pub enum Op1Def {
    Constant(f32),
    Pulser(Pulser),
    Brightness(u32), // op3
    Sum(Vec<u32>), // op1...
}

pub enum Op3Def {
    Constant(Pix<f32>),
    Grey(u32), // op1
    RGB(u32, u32, u32), // op1, op1, op1
    Sum(Vec<u32>), // op3...
}

pub struct Op1 {
    def: Op1Def,
    buf: Vec<f32>,
}

pub struct Op3 {
    def: Op3Def,
    buf: Vec<Pix<f32>>,
}

pub enum ScriptIndex {
    Op1(u32),
    Op3(u32),
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

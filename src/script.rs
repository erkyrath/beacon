use crate::context::RunContext;
use crate::pixel::Pix;
use crate::op::{Op1Def, Op3Def};
use crate::pulser::Pulser;

pub enum ScriptIndex {
    Op1(usize),
    Op3(usize),
}

pub struct Script {
    pub order: Vec<ScriptIndex>, // 0 is root
    pub op1s: Vec<Op1Def>,
    pub op3s: Vec<Op3Def>,
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

pub fn build_script() -> Script {
    let mut script = Script::new();

    /*
    let inverter = Op1Def::Invert(1);
    script.order.push(ScriptIndex::Op1(script.op1s.len()));
    script.op1s.push(inverter);
    */
    
    let pulser = Op1Def::Pulser(Pulser::new());
    script.order.push(ScriptIndex::Op1(script.op1s.len()));
    script.op1s.push(pulser);

    return script;
}

use crate::param::Param;
use crate::op::{Op1Def, Op3Def};
use crate::pixel::Pix;

use crate::pulser::Pulser;
use crate::waves::WaveShape;

#[derive(Copy, Clone)]
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

    let cmuls = Op3Def::CMulS(1, 0);
    script.order.push(ScriptIndex::Op3(script.op3s.len()));
    script.op3s.push(cmuls);

    let cconst = Op3Def::Constant(Pix::new(1.0, 0.2, 1.0));
    script.order.push(ScriptIndex::Op3(script.op3s.len()));
    script.op3s.push(cconst);

    let mut pulserdef = Pulser::new();
    //pulserdef.pos = Param::Quote(Box::new(Param::Changing(1.2, -0.4)));
    pulserdef.pos = Param::Quote(Box::new(Param::Wave(WaveShape::SqrTooth, 0.1, 0.9, 2.0)));
    //pulserdef.width = Param::Quote(Box::new(Param::Changing(0.2, 0.2)));
    pulserdef.width = Param::Constant(0.2);
    pulserdef.spaceshape = WaveShape::Sine;
    pulserdef.timeshape = WaveShape::Flat;
    pulserdef.duration = Param::Constant(3.0);
    pulserdef.interval = Param::Constant(0.5);
    pulserdef.countlimit = Some(1);
    
    let pulser = Op1Def::Pulser(pulserdef);
    script.order.push(ScriptIndex::Op1(script.op1s.len()));
    script.op1s.push(pulser);

    return script;
}

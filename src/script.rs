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

    pub fn dump(&self) {
        if self.order.len() == 0 {
            println!("script is empty");
            return;
        }

        self.dumpop(self.order[0], 0);
    }
    
    fn dumpop(&self, scix: ScriptIndex, indent: usize) {
        let indentstr: String = vec!["  "; indent].join("");
        let subindentstr = "\n         ".to_string() + &indentstr;
        let desc: String;
        let bufs: Vec<ScriptIndex>;
        let scstr: String;
        match scix {
            ScriptIndex::Op1(bufnum) => {
                (desc, bufs) = self.op1s[bufnum].describe(Some(subindentstr));
                scstr = format!("1/{}", bufnum);
            },
            ScriptIndex::Op3(bufnum) => {
                (desc, bufs) = self.op3s[bufnum].describe(Some(subindentstr));
                scstr = format!("3/{}", bufnum);
            },
        }

        println!("({}): {}{}", scstr, indentstr, desc);
        
        for val in bufs {
            self.dumpop(val, indent+1);
        }
    }
    
}

pub fn build_script() -> Script {
    let mut script = Script::new();

    let cmuls = Op3Def::CMulS(1, 0);
    script.order.push(ScriptIndex::Op3(script.op3s.len()));
    script.op3s.push(cmuls);

    let cconst = Op3Def::Constant(Pix::new(0.5, 0.2, 0.5));
    script.order.push(ScriptIndex::Op3(script.op3s.len()));
    script.op3s.push(cconst);

    let mut pulserdef = Pulser::new();
    //pulserdef.pos = Param::Quote(Box::new(Param::Changing(1.2, -0.4)));
    pulserdef.pos = Param::Quote(Box::new(Param::WaveCycle(WaveShape::Sine, 0.1, 0.9, 3.0)));
    //pulserdef.width = Param::Quote(Box::new(Param::Changing(0.2, 0.2)));
    pulserdef.width = Param::Constant(0.2);
    pulserdef.spaceshape = WaveShape::Sine;
    pulserdef.timeshape = WaveShape::Flat;
    pulserdef.duration = Param::Constant(3.0);
    pulserdef.interval = Param::Constant(0.5);
    pulserdef.countlimit = Some(4);
    
    let pulser = Op1Def::Pulser(pulserdef);
    script.order.push(ScriptIndex::Op1(script.op1s.len()));
    script.op1s.push(pulser);

    return script;
}

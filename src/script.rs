use std::collections::HashSet;

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

struct BufTrackPair {
    op1s: HashSet<usize>,
    op3s: HashSet<usize>,
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
        let mut track = BufTrackPair {
            op1s: HashSet::new(),
            op3s: HashSet::new(),
        };
        
        if self.order.len() == 0 {
            println!("script has no root");
        }
        else {
            self.dumpop(&mut track, self.order[0], 0);
        }

        // Now any unmentioned ops
        for bufnum in 0..self.op3s.len() {
            if !track.op3s.contains(&bufnum) {
                self.dumpop(&mut track, ScriptIndex::Op3(bufnum), 0);
            }
        }
        for bufnum in 0..self.op1s.len() {
            if !track.op1s.contains(&bufnum) {
                self.dumpop(&mut track, ScriptIndex::Op1(bufnum), 0);
            }
        }
    }
    
    fn dumpop(&self, track: &mut BufTrackPair, scix: ScriptIndex, indent: usize) {
        let indentstr: String = "  ".repeat(indent);
        let subindentstr = "\n         ".to_string() + &indentstr;
        let desc: String;
        let bufs: Vec<ScriptIndex>;
        let scstr: String;
        match scix {
            ScriptIndex::Op1(bufnum) => {
                if bufnum < self.op1s.len() {
                    track.op1s.insert(bufnum);
                    (desc, bufs) = self.op1s[bufnum].describe(Some(subindentstr));
                }
                else {
                    desc = "???".to_string();
                    bufs = Vec::default();
                }
                scstr = format!("1/{}", bufnum);
            },
            ScriptIndex::Op3(bufnum) => {
                if bufnum < self.op3s.len() {
                    track.op3s.insert(bufnum);
                    (desc, bufs) = self.op3s[bufnum].describe(Some(subindentstr));
                }
                else {
                    desc = "???".to_string();
                    bufs = Vec::default();
                }
                scstr = format!("3/{}", bufnum);
            },
        }

        println!("({}): {}{}", scstr, indentstr, desc);
        
        for val in bufs {
            self.dumpop(track, val, indent+1);
        }
    }
    
}

pub fn build_script() -> Script {
    let mut script = Script::new();

    let csum = Op3Def::Sum(vec![3, 1]);
    script.order.push(ScriptIndex::Op3(script.op3s.len()));
    script.op3s.push(csum);

    let cmuls = Op3Def::MulS(2, 0);
    script.order.push(ScriptIndex::Op3(script.op3s.len()));
    script.op3s.push(cmuls);

    let cconst = Op3Def::Constant(Pix::new(0.7, 0.2, 0.9));
    script.order.push(ScriptIndex::Op3(script.op3s.len()));
    script.op3s.push(cconst);

    let bconst = Op3Def::Constant(Pix::new(0.0, 0.4, 0.0));
    script.order.push(ScriptIndex::Op3(script.op3s.len()));
    script.op3s.push(bconst);

    let mut pulserdef = Pulser::new();
    //pulserdef.pos = Param::Quote(Box::new(Param::Changing(1.2, -0.4)));
    pulserdef.pos = Param::Quote(Box::new(Param::WaveCycle(WaveShape::Sine, 0.1, 0.9, 3.0)));
    //pulserdef.width = Param::Quote(Box::new(Param::Changing(0.2, 0.2)));
    pulserdef.width = Param::Constant(0.2);
    pulserdef.spaceshape = WaveShape::Sine;
    pulserdef.timeshape = WaveShape::Flat;
    pulserdef.duration = Param::Constant(3.0);
    pulserdef.interval = Param::Constant(0.5);
    pulserdef.countlimit = Some(2);
    
    let pulser = Op1Def::Pulser(pulserdef);
    script.order.push(ScriptIndex::Op1(script.op1s.len()));
    script.op1s.push(pulser);

    return script;
}

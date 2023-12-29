use std::collections::HashSet;

use crate::param::Param;
use crate::op::{Op1Def, Op3Def};
use crate::pixel::Pix;

use crate::pulser::Pulser;
use crate::waves::WaveShape;

#[derive(Copy, Clone, Debug)]
pub enum ScriptIndex {
    Op1(usize),
    Op3(usize),
}

pub struct Op1DefRef {
    pub op: Op1Def,
    pub bufs: Vec<ScriptIndex>,
}

pub struct Op3DefRef {
    pub op: Op3Def,
    pub bufs: Vec<ScriptIndex>,
}

impl Op1DefRef {
    pub fn new(op: Op1Def, bufs: Vec<ScriptIndex>) -> Op1DefRef {
        Op1DefRef { op:op, bufs:bufs }
    }
    
    pub fn get_type_ref(&self, op: u8, num: usize) -> usize {
        if op == 1 {
            if let ScriptIndex::Op1(val) = self.bufs[num] {
                return val;
            }
        } else if op == 3 {
            if let ScriptIndex::Op3(val) = self.bufs[num] {
                return val;
            }
        }
        panic!("invalid typeref: type {} num {}", op, num);
    }
}

impl Op3DefRef {
    pub fn new(op: Op3Def, bufs: Vec<ScriptIndex>) -> Op3DefRef {
        Op3DefRef { op:op, bufs:bufs }
    }
    
    pub fn get_type_ref(&self, op: u8, num: usize) -> usize {
        if op == 1 {
            if let ScriptIndex::Op1(val) = self.bufs[num] {
                return val;
            }
        } else if op == 3 {
            if let ScriptIndex::Op3(val) = self.bufs[num] {
                return val;
            }
        }
        panic!("invalid typeref: type {} num {}", op, num);
    }
}

pub struct Script {
    pub order: Vec<ScriptIndex>, // 0 is root
    pub op1s: Vec<Op1DefRef>,
    pub op3s: Vec<Op3DefRef>,
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
            println!("script order is empty");
        }
        else {
            println!("script order: {:?}", self.order);
            self.dumpop(&mut track, self.order[0], 0);
        }

        // Now any unmentioned ops
        let mut gotany = false;
        for bufnum in 0..self.op3s.len() {
            if !track.op3s.contains(&bufnum) {
                if !gotany {
                    gotany = true;
                    println!("unmentioned ops:");
                }
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
        let bufs: Option<&Vec<ScriptIndex>>;
        let scstr: String;
        match scix {
            ScriptIndex::Op1(bufnum) => {
                if bufnum < self.op1s.len() {
                    track.op1s.insert(bufnum);
                    bufs = Some(&self.op1s[bufnum].bufs);
                    desc = self.op1s[bufnum].op.describe(Some(subindentstr));
                }
                else {
                    desc = "???".to_string();
                    bufs = None;
                }
                scstr = format!("1/{}", bufnum);
            },
            ScriptIndex::Op3(bufnum) => {
                if bufnum < self.op3s.len() {
                    track.op3s.insert(bufnum);
                    bufs = Some(&self.op3s[bufnum].bufs);
                    desc = self.op3s[bufnum].op.describe(Some(subindentstr));
                }
                else {
                    desc = "???".to_string();
                    bufs = None;
                }
                scstr = format!("3/{}", bufnum);
            },
        }

        println!("({}): {}{}", scstr, indentstr, desc);

        if let Some(buflist) = bufs {
            for val in buflist {
                self.dumpop(track, *val, indent+1);
            }
        }
    }
    
}

pub fn build_script() -> Script {
    let mut script = Script::new();

    let csum = Op3Def::Sum();
    let csumbufs: Vec<ScriptIndex> = vec![ ScriptIndex::Op3(3), ScriptIndex::Op3(1) ];
    script.order.push(ScriptIndex::Op3(script.op3s.len()));
    script.op3s.push(Op3DefRef::new(csum, csumbufs));

    let cmuls = Op3Def::MulS();
    let cmulsbufs: Vec<ScriptIndex> = vec![ ScriptIndex::Op3(2), ScriptIndex::Op1(0) ];
    script.order.push(ScriptIndex::Op3(script.op3s.len()));
    script.op3s.push(Op3DefRef::new(cmuls, cmulsbufs));

    let cconst = Op3Def::Constant(Pix::new(0.7, 0.2, 0.9));
    script.order.push(ScriptIndex::Op3(script.op3s.len()));
    script.op3s.push(Op3DefRef::new(cconst, Vec::default()));

    let bconst = Op3Def::Constant(Pix::new(0.0, 0.4, 0.0));
    script.order.push(ScriptIndex::Op3(script.op3s.len()));
    script.op3s.push(Op3DefRef::new(bconst, Vec::default()));

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
    script.op1s.push(Op1DefRef::new(pulser, Vec::default()));

    return script;
}

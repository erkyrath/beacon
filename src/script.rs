use std::collections::HashSet;

use crate::param::Param;
use crate::op::{Op1Def, Op3Def};
use crate::pixel::Pix;

use crate::pulser::Pulser;
use crate::waves::WaveShape;

#[derive(Copy, Clone, Debug, PartialEq)]
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

    pub fn consistency_check(&self) -> Result<(), String> {
        for ix in 0..self.order.len() {
            let scix = &self.order[ix];
            let buflist = match scix {
                ScriptIndex::Op1(bufnum) => {
                    &self.op1s.get(*bufnum)
                        .ok_or_else(|| format!("SceneIndex {:?} does not exist", scix))?
                        .bufs
                },
                ScriptIndex::Op3(bufnum) => {
                    &self.op3s.get(*bufnum)
                        .ok_or_else(|| format!("SceneIndex {:?} does not exist", scix))?
                        .bufs
                },
            };
            for scjx in buflist {
                if let Some(pos) = self.order.iter().position(|val| val == scjx) {
                    if pos <= ix {
                        return Err(format!("SceneIndex {:?} refers to {:?} which is earlier in the order", scix, scjx));
                    }
                }
                else {
                    return Err(format!("SceneIndex {:?} refers to {:?} which does not exist", scix, scjx));
                }
            }
        }

        Ok(())
    }
    
    pub fn dump(&self) {
        let mut track = BufTrackPair {
            op1s: HashSet::new(),
            op3s: HashSet::new(),
        };
        
        println!("script has {} 1-bufs, {} 3-bufs", self.op1s.len(), self.op3s.len());
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
                if !gotany {
                    gotany = true;
                    println!("unmentioned ops:");
                }
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
                if track.op1s.contains(&bufnum) {
                    desc = "(var)".to_string(); //### varname
                    bufs = None;
                }
                else if bufnum < self.op1s.len() {
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
                if track.op3s.contains(&bufnum) {
                    desc = "(var)".to_string(); //### varname
                    bufs = None;
                }
                else if bufnum < self.op3s.len() {
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


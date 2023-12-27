use std::fmt;
use std::collections::HashMap;
use lazy_static::lazy_static;

use crate::op::{Op1Def, Op3Def};
use crate::pixel::Pix;
use crate::parse::tree::{ParseTerm, ParseNode};

pub mod tree;

//###?
enum OpLayoutType {
    Op1,
    Op3,
    Number,
    Color,
    Param,
    Wave,
}

struct OpLayoutParam {
    name: String,
    ptype: OpLayoutType,
    optional: bool,
}

lazy_static! {
    static ref PARAMLAYOUT: HashMap<&'static str, Vec<OpLayoutParam>> = {
        let mut map = HashMap::new();
        map.insert("constant", vec![
            OpLayoutParam { name: "_1".to_string(), ptype: OpLayoutType::Number, optional: false },
        ]);
        map.insert("randflat", vec![
            OpLayoutParam { name: "min".to_string(), ptype: OpLayoutType::Number, optional: false },
            OpLayoutParam { name: "max".to_string(), ptype: OpLayoutType::Number, optional: false },
        ]);
        map.insert("randnorm", vec![
            OpLayoutParam { name: "mean".to_string(), ptype: OpLayoutType::Number, optional: true },
            OpLayoutParam { name: "stdev".to_string(), ptype: OpLayoutType::Number, optional: true },
        ]);
        map.insert("changing", vec![
            OpLayoutParam { name: "start".to_string(), ptype: OpLayoutType::Number, optional: false },
            OpLayoutParam { name: "velocity".to_string(), ptype: OpLayoutType::Number, optional: false },
        ]);
        map.insert("wave", vec![
            OpLayoutParam { name: "shape".to_string(), ptype: OpLayoutType::Wave, optional: false },
            OpLayoutParam { name: "min".to_string(), ptype: OpLayoutType::Number, optional: true },
            OpLayoutParam { name: "max".to_string(), ptype: OpLayoutType::Number, optional: true },
            OpLayoutParam { name: "duration".to_string(), ptype: OpLayoutType::Number, optional: true },
        ]);
        map.insert("wavecycle", vec![
            OpLayoutParam { name: "shape".to_string(), ptype: OpLayoutType::Wave, optional: false },
            OpLayoutParam { name: "min".to_string(), ptype: OpLayoutType::Number, optional: true },
            OpLayoutParam { name: "max".to_string(), ptype: OpLayoutType::Number, optional: true },
            OpLayoutParam { name: "period".to_string(), ptype: OpLayoutType::Number, optional: true },
        ]);
        map
    };
    
    static ref OP3LAYOUT: HashMap<&'static str, Vec<OpLayoutParam>> = {
        let mut map = HashMap::new();
        map.insert("invert", vec![
            OpLayoutParam { name: "_1".to_string(), ptype: OpLayoutType::Op3, optional: false },
        ]);
        map.insert("grey", vec![
            OpLayoutParam { name: "_1".to_string(), ptype: OpLayoutType::Op1, optional: false },
        ]);
        map
    };
}

struct BuildOp1 {
    op1: Option<Box<Op1Def>>,
    child1: Vec<Box<BuildOp1>>,
    child3: Vec<Box<BuildOp3>>,
}

struct BuildOp3 {
    op3: Option<Box<Op3Def>>,
    child1: Vec<Box<BuildOp1>>,
    child3: Vec<Box<BuildOp3>>,
}

impl BuildOp1 {
    fn new(op: Op1Def) -> BuildOp1 {
        BuildOp1 {
            op1: Some(Box::new(op)),
            child1: Vec::default(),
            child3: Vec::default(),
        }
    }

    fn addchild1(&mut self, op: Op1Def) {
        self.child1.push(Box::new(BuildOp1::new(op)));
    }

    fn addchild3(&mut self, op: Op3Def) {
        self.child3.push(Box::new(BuildOp3::new(op)));
    }
}

impl BuildOp3 {
    fn new(op: Op3Def) -> BuildOp3 {
        BuildOp3 {
            op3: Some(Box::new(op)),
            child1: Vec::default(),
            child3: Vec::default(),
        }
    }

    fn addchild1(mut self, op: Op1Def) -> BuildOp3 {
        self.child1.push(Box::new(BuildOp1::new(op)));
        return self;
    }

    fn addchild3(mut self, op: Op3Def) -> BuildOp3 {
        self.child3.push(Box::new(BuildOp3::new(op)));
        return self;
    }
}

impl fmt::Debug for BuildOp1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.op1 {
            None => write!(f, "(none)")?,
            Some(opdef) => opdef.fmt(f)?,
        }
        
        let mut gotany = false;
        for subop in &self.child1 {
            if !gotany { write!(f, "[")?; }
            else { write!(f, ", ")?; }
            subop.fmt(f)?;
            gotany = true;
        }
        for subop in &self.child3 {
            if !gotany { write!(f, "[")?; }
            else { write!(f, ", ")?; }
            subop.fmt(f)?;
            gotany = true;
        }
        if gotany { write!(f, "]")?; }

        Ok(())
    }
}

impl fmt::Debug for BuildOp3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.op3 {
            None => write!(f, "(none)")?,
            Some(opdef) => opdef.fmt(f)?,
        }
        
        let mut gotany = false;
        for subop in &self.child1 {
            if !gotany { write!(f, "[")?; }
            else { write!(f, ", ")?; }
            subop.fmt(f)?;
            gotany = true;
        }
        for subop in &self.child3 {
            if !gotany { write!(f, "[")?; }
            else { write!(f, ", ")?; }
            subop.fmt(f)?;
            gotany = true;
        }
        if gotany { write!(f, "]")?; }
        
        Ok(())
    }
}

pub fn parse_script(filename: &str) -> Result<(), String> {
    let itemls = tree::parse_tree(filename)?;

    for item in &itemls.items {
        let op3 = parse_for_op3(item)?;
        println!("### got op3 {:?}", op3);
    }
    
    return Ok(());
}

fn parse_for_op3(nod: &ParseNode) -> Result<BuildOp3, String> {
    match &nod.term {
        ParseTerm::Color(pix) => {
            verify_childless(nod)?;
            let op = Op3Def::Constant(pix.clone());
            Ok(BuildOp3::new(op))
        },
        ParseTerm::Number(val) => {
            verify_childless(nod)?;
            let subop = Op1Def::Constant(*val);
            let op = Op3Def::Grey(0);
            Ok(BuildOp3::new(op).addchild1(subop))
        },
        ParseTerm::Ident(val) => {
            let params = OP3LAYOUT.get(val.to_lowercase().as_str())
                .ok_or_else(|| format!("line {}: op3 not recognized: {}", nod.linenum, val))?;
            let pmap = match_children(nod, params)?;
            println!("### pmap = {:?}", pmap);
            return Err("### TODO".to_string()); //###
            
            /*###
            match val.to_lowercase().as_str() {
                "grey" => {
                    let subop = Op1Def::Constant(0.123); //###
                    let op = Op3Def::Grey(0);
                    Ok(BuildOp3::new(op).addchild1(subop))
                },
                "invert" => {
                    let subop = Op3Def::Constant(Pix::new(0.123, 0.123, 0.123)); //###
                    let op = Op3Def::Invert(0);
                    Ok(BuildOp3::new(op).addchild3(subop))
                },
                _ => {
                    Err(format!("line {}: op3 not recognized: {}", nod.linenum, val))
                },
            }
            ###*/
        },
        //_ => Err(format!("unimplemented at line {}", nod.linenum)),
    }
}

fn verify_childless(nod: &ParseNode) -> Result<(), String> {
    if nod.params.items.len() > 0 {
        return Err(format!("line {}: node cannot have params: {:?}", nod.linenum, nod.term));
    }
    Ok(())
}

fn match_children(nod: &ParseNode, layout: &Vec<OpLayoutParam>) -> Result<HashMap<String, usize>, String> {
    let mut res: HashMap<String, usize> = HashMap::new();
    let mut used = vec![false; layout.len()];

    for item in &nod.params.items {
        match &item.key {
            None => {
                if let Some(pos) = used.iter().position(|val|!val) {
                    used[pos] = true;
                    res.insert(layout[pos].name.clone(), pos);
                }
                else {
                    return Err(format!("line {}: too many params", nod.linenum));
                }
            },
            Some(name) => {
                if let Some(pos) = layout.iter().position(|val| &val.name == name) {
                    if used[pos] {
                        return Err(format!("line {}: param appears twice: {}", nod.linenum, name));
                    }
                    else {
                        used[pos] = true;
                        res.insert(layout[pos].name.clone(), pos);
                    }
                }
                else {
                    return Err(format!("line {}: param not known: {}", nod.linenum, name));
                }
            },
        }
    }
    
    Ok(res)
}

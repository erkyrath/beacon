use std::fmt;

use crate::op::{Op1Def, Op3Def};
use crate::pixel::Pix;
use crate::parse::tree::{ParseTerm, ParseNode};

pub mod tree;

//###?
enum TreeCtx {
    Op1,
    Op3,
    Number,
    Color,
    Param,
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
}

impl BuildOp3 {
    fn new(op: Op3Def) -> BuildOp3 {
        BuildOp3 {
            op3: Some(Box::new(op)),
            child1: Vec::default(),
            child3: Vec::default(),
        }
    }
}

impl fmt::Debug for BuildOp1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.op1 {
            None => write!(f, "(none)"),
            Some(opdef) => opdef.fmt(f),
        }
    }
}

impl fmt::Debug for BuildOp3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.op3 {
            None => write!(f, "(none)"),
            Some(opdef) => opdef.fmt(f),
        }
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
            //### verify no children
            let op = Op3Def::Constant(pix.clone());
            Ok(BuildOp3::new(op))
        },
        ParseTerm::Number(val) => {
            //### verify no children
            //### Grey(Constant(val)) would be better
            let op = Op3Def::Constant(Pix::new(*val, *val, *val));
            Ok(BuildOp3::new(op))
        },
        _ => Err(format!("unimplemented at line {}", nod.linenum)),
    }
}


pub mod tree;
pub mod layout;

use std::fmt;
use std::collections::HashMap;
use lazy_static::lazy_static;

use crate::op::{Op1Def, Op3Def};
use crate::pixel::Pix;
use crate::parse::tree::{ParseTerm, ParseNode};
use crate::parse::layout::{OpLayoutParam};
use crate::parse::layout::{get_param_layout, get_op1_layout, get_op3_layout};

pub struct BuildOp1 {
    op1: Option<Box<Op1Def>>,
    child1: Vec<Box<BuildOp1>>,
    child3: Vec<Box<BuildOp3>>,
}

pub struct BuildOp3 {
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

    fn addchild1(mut self, op: BuildOp1) -> BuildOp1 {
        self.child1.push(Box::new(op));
        return self;
    }

    fn addchild3(mut self, op: BuildOp3) -> BuildOp1 {
        self.child3.push(Box::new(op));
        return self;
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

    fn addchild1(mut self, op: BuildOp1) -> BuildOp3 {
        self.child1.push(Box::new(op));
        return self;
    }

    fn addchild3(mut self, op: BuildOp3) -> BuildOp3 {
        self.child3.push(Box::new(op));
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
        match parse_for_op3(item) {
            Ok(op3) => {
                println!("### got op3 {:?}", op3);
            },
            Err(err3) => {
                match parse_for_op1(item) {
                    Ok(op1) => {
                        println!("### got op1 {:?}", op1);
                    },
                    Err(_err1) => {
                        return Err(err3);
                    }
                }
            },
        }
    }
    
    return Ok(());
}

fn parse_for_number(nod: &ParseNode) -> Result<f32, String> {
    match &nod.term {
        ParseTerm::Number(val) => {
            verify_childless(nod)?;
            Ok(*val)
        },
        _ => Err(format!("line {}: number expected", nod.linenum)),
    }
}

fn parse_for_color(nod: &ParseNode) -> Result<Pix<f32>, String> {
    match &nod.term {
        ParseTerm::Color(pix) => {
            verify_childless(nod)?;
            Ok(pix.clone())
        },
        _ => Err(format!("line {}: color expected", nod.linenum)),
    }
}

fn parse_for_op1(nod: &ParseNode) -> Result<BuildOp1, String> {
    match &nod.term {
        ParseTerm::Color(_pix) => {
            Err(format!("line {}: unexpected color", nod.linenum))
        },
        ParseTerm::Number(val) => {
            verify_childless(nod)?;
            let op = Op1Def::Constant(*val);
            Ok(BuildOp1::new(op))
        },
        ParseTerm::Ident(val) => {
            let (params, buildfunc) = get_op1_layout(val)
                .ok_or_else(|| format!("line {}: op1 not recognized: {}", nod.linenum, val))?;
            let pmap = match_children(nod, params)?;
            println!("### pmap = {:?}", pmap);
            return buildfunc(nod, &pmap);
        },
        //_ => Err(format!("unimplemented at line {}", nod.linenum)),
    }
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
            Ok(BuildOp3::new(op).addchild1(BuildOp1::new(subop)))
        },
        ParseTerm::Ident(val) => {
            let (params, buildfunc) = get_op3_layout(val)
                .ok_or_else(|| format!("line {}: op3 not recognized: {}", nod.linenum, val))?;
            let pmap = match_children(nod, params)?;
            println!("### pmap = {:?}", pmap);
            return buildfunc(nod, &pmap);
        },
        //_ => Err(format!("unimplemented at line {}", nod.linenum)),
    }
}

fn verify_childless(nod: &ParseNode) -> Result<(), String> {
    if nod.params.items.len() > 0 {
        return Err(format!("line {}: node cannot have params: {}", nod.linenum, nod.term));
    }
    Ok(())
}

fn match_children(nod: &ParseNode, layout: &Vec<OpLayoutParam>) -> Result<HashMap<String, usize>, String> {
    let mut res: HashMap<String, usize> = HashMap::new();
    let mut used = vec![false; layout.len()];

    for (itemix, item) in nod.params.items.iter().enumerate() {
        match &item.key {
            None => {
                if let Some(pos) = used.iter().position(|val|!val) {
                    used[pos] = true;
                    res.insert(layout[pos].name.clone(), itemix);
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
                        res.insert(layout[pos].name.clone(), itemix);
                    }
                }
                else {
                    return Err(format!("line {}: param not known for {}: {}", nod.linenum, nod.term, name));
                }
            },
        }
    }

    for pos in 0..layout.len() {
        if !used[pos] && !layout[pos].optional {
            return Err(format!("line {}: required parameter for {}: {}", nod.linenum, nod.term, layout[pos].name));
        }
    }
    
    Ok(res)
}

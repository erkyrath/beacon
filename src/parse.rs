pub mod tree;
pub mod layout;

use std::fmt;
use std::collections::HashMap;
use lazy_static::lazy_static;

use crate::op::{Op1Def, Op3Def};
use crate::pixel::Pix;
use crate::waves::WaveShape;
use crate::param::Param;
use crate::script::{Script, ScriptIndex};
use crate::script::{Op1DefRef, Op3DefRef};
use crate::parse::tree::{ParseTerm, ParseNode};
use crate::parse::layout::{OpLayoutParam};
use crate::parse::layout::{get_waveshape, get_param_layout, get_op1_layout, get_op3_layout};

enum BuildOp {
    Op1(Box<BuildOp1>),
    Op3(Box<BuildOp3>),
}

impl fmt::Debug for BuildOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildOp::Op1(op) => op.fmt(f),
            BuildOp::Op3(op) => op.fmt(f),
        }
    }
}

pub struct BuildOp1 {
    op1: Option<Box<Op1Def>>,
    children: Vec<BuildOp>,
}

pub struct BuildOp3 {
    op3: Option<Box<Op3Def>>,
    children: Vec<BuildOp>,
}

impl BuildOp1 {
    fn new(op: Op1Def) -> BuildOp1 {
        BuildOp1 {
            op1: Some(Box::new(op)),
            children: Vec::default(),
        }
    }

    fn addchild1(mut self, op: BuildOp1) -> BuildOp1 {
        self.children.push(BuildOp::Op1(Box::new(op)));
        return self;
    }

    fn addchild3(mut self, op: BuildOp3) -> BuildOp1 {
        self.children.push(BuildOp::Op3(Box::new(op)));
        return self;
    }

    fn build(&self, script: &mut Script) -> ScriptIndex {
        let mut bufs: Vec<ScriptIndex> = Vec::default();
        for bnod in &self.children {
            match bnod {
                BuildOp::Op1(nod) => {
                    let obufnum = nod.build(script);
                    bufs.push(obufnum);
                },
                BuildOp::Op3(nod) => {
                    let obufnum = nod.build(script);
                    bufs.push(obufnum);
                },
            }
        }
        let bufnum = script.op1s.len();
        script.order.push(ScriptIndex::Op1(bufnum));
        if let Some(op) = &self.op1 {
            script.op1s.push(Op1DefRef::new(*op.clone(), bufs));
        }
        else {
            panic!("build: missing opdef1");
        }
        return ScriptIndex::Op1(bufnum);
    }
        
}

impl BuildOp3 {
    fn new(op: Op3Def) -> BuildOp3 {
        BuildOp3 {
            op3: Some(Box::new(op)),
            children: Vec::default(),
        }
    }

    fn addchild1(mut self, op: BuildOp1) -> BuildOp3 {
        self.children.push(BuildOp::Op1(Box::new(op)));
        return self;
    }

    fn addchild3(mut self, op: BuildOp3) -> BuildOp3 {
        self.children.push(BuildOp::Op3(Box::new(op)));
        return self;
    }

    fn build(&self, script: &mut Script) -> ScriptIndex {
        let mut bufs: Vec<ScriptIndex> = Vec::default();
        for bnod in &self.children {
            match bnod {
                BuildOp::Op1(nod) => {
                    let obufnum = nod.build(script);
                    bufs.push(obufnum);
                },
                BuildOp::Op3(nod) => {
                    let obufnum = nod.build(script);
                    bufs.push(obufnum);
                },
            }
        }
        let bufnum = script.op3s.len();
        script.order.push(ScriptIndex::Op3(bufnum));
        if let Some(op) = &self.op3 {
            script.op3s.push(Op3DefRef::new(*op.clone(), bufs));
        }
        else {
            panic!("build: missing opdef3");
        }
        return ScriptIndex::Op3(bufnum);
    }
        
}

impl fmt::Debug for BuildOp1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.op1 {
            None => write!(f, "(none)")?,
            Some(opdef) => opdef.fmt(f)?,
        }
        
        let mut gotany = false;
        for subop in &self.children {
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
        for subop in &self.children {
            if !gotany { write!(f, "[")?; }
            else { write!(f, ", ")?; }
            subop.fmt(f)?;
            gotany = true;
        }
        if gotany { write!(f, "]")?; }
        
        Ok(())
    }
}

pub fn parse_script(filename: &str) -> Result<Script, String> {
    let itemls = tree::parse_tree(filename)?;

    let mut script = Script::new();

    for item in &itemls.items {
        verify_wellformed(&item, 0)?;
    }

    for item in &itemls.items {
        //### this gives a bad error if pulser is the root
        match parse_for_op3(item) {
            Ok(op3) => {
                //println!("got op3 (name {:?}) {:?}", item.key, op3);
                op3.build(&mut script);
            },
            Err(err3) => {
                match parse_for_op1(item) {
                    Ok(op1) => {
                        //println!("got op1 (name {:?}) {:?}", item.key, op1);
                        op1.build(&mut script);
                    },
                    Err(_err1) => {
                        return Err(err3);
                    }
                }
            },
        }
    }

    if script.order.len() == 0 {
        return Err("error: script is empty".to_string());
    }
    
    script.order.reverse();
    
    return Ok(script);
}

fn verify_wellformed(nod: &ParseNode, depth: usize) -> Result<(), String> {
    match &nod.term {
        ParseTerm::Number(_val) => {
            if nod.params.items.len() > 0 {
                return Err(format!("line {}: number cannot have params: {}", nod.linenum, nod.term));
            }
        },
        ParseTerm::Color(_val) => {
            if nod.params.items.len() > 0 {
                return Err(format!("line {}: color cannot have params: {}", nod.linenum, nod.term));
            }
        },
        ParseTerm::VarName(_val) => {
            if nod.params.items.len() > 0 {
                return Err(format!("line {}: variable ref cannot have params: {}", nod.linenum, nod.term));
            }
            //### check that var exists
        },
        ParseTerm::Ident(_val) => {
            for item in &nod.params.items {
                verify_wellformed(item, depth+1)?;
            }
        },
    }
    Ok(())
}

fn parse_for_number(nod: &ParseNode) -> Result<f32, String> {
    match &nod.term {
        ParseTerm::Number(val) => {
            Ok(*val)
        },
        _ => Err(format!("line {}: number expected", nod.linenum)),
    }
}

fn parse_for_color(nod: &ParseNode) -> Result<Pix<f32>, String> {
    match &nod.term {
        ParseTerm::Color(pix) => {
            Ok(pix.clone())
        },
        _ => Err(format!("line {}: color expected", nod.linenum)),
    }
}

fn parse_for_waveshape(nod: &ParseNode) -> Result<WaveShape, String> {
    match &nod.term {
        ParseTerm::Ident(val) => {
            verify_childless(nod)?;
            match get_waveshape(val) {
                Some(shape) => Ok(*shape),
                _ => Err(format!("line {}: waveshape expected", nod.linenum)),
            }
        },
        _ => Err(format!("line {}: waveshape expected", nod.linenum)),
    }
}

fn parse_for_param(nod: &ParseNode) -> Result<Param, String> {
    match &nod.term {
        ParseTerm::Color(_pix) => {
            Err(format!("line {}: unexpected color", nod.linenum))
        },
        ParseTerm::Number(val) => {
            Ok(Param::Constant(*val))
        },
        ParseTerm::VarName(_val) => {
            Err(format!("line {}: param cannot be variable ref", nod.linenum))
        },
        ParseTerm::Ident(val) => {
            let (params, buildfunc) = get_param_layout(val)
                .ok_or_else(|| format!("line {}: param not recognized: {}", nod.linenum, val))?;
            let pmap = match_children(nod, params)?;
            return buildfunc(nod, &pmap);
        },
        //_ => Err(format!("unimplemented at line {}", nod.linenum)),
    }
}

fn parse_for_op1(nod: &ParseNode) -> Result<BuildOp1, String> {
    match &nod.term {
        ParseTerm::Color(_pix) => {
            Err(format!("line {}: unexpected color", nod.linenum))
        },
        ParseTerm::Number(val) => {
            let op = Op1Def::Constant(*val);
            Ok(BuildOp1::new(op))
        },
        ParseTerm::VarName(_val) => {
            panic!("###");
        },
        ParseTerm::Ident(val) => {
            let (params, buildfunc) = get_op1_layout(val)
                .ok_or_else(|| format!("line {}: op1 not recognized: {}", nod.linenum, val))?;
            let pmap = match_children(nod, params)?;
            return buildfunc(nod, &pmap);
        },
        //_ => Err(format!("unimplemented at line {}", nod.linenum)),
    }
}

fn parse_for_op3(nod: &ParseNode) -> Result<BuildOp3, String> {
    match &nod.term {
        ParseTerm::Color(pix) => {
            let op = Op3Def::Constant(pix.clone());
            Ok(BuildOp3::new(op))
        },
        ParseTerm::Number(val) => {
            let subop = Op1Def::Constant(*val);
            let op = Op3Def::Grey();
            Ok(BuildOp3::new(op).addchild1(BuildOp1::new(subop)))
        },
        ParseTerm::VarName(_val) => {
            panic!("###");
        },
        ParseTerm::Ident(val) => {
            let (params, buildfunc) = get_op3_layout(val)
                .ok_or_else(|| format!("line {}: op3 not recognized: {}", nod.linenum, val))?;
            let pmap = match_children(nod, params)?;
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
    let mut repcount: HashMap<String, usize> = HashMap::new();

    for (itemix, item) in nod.params.items.iter().enumerate() {
        match &item.key {
            None => {
                if let Some(pos) = used.iter().position(|val|!val) {
                    if !layout[pos].repeating {
                        used[pos] = true;
                        res.insert(layout[pos].name.clone(), itemix);
                    }
                    else {
                        let count = repcount.entry(layout[pos].name.clone())
                            .and_modify(|val| { *val += 1 })
                            .or_insert(1);
                        let tempname = format!("{}{}", layout[pos].name, count);
                        res.insert(tempname, itemix);
                    }
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
                        if !layout[pos].repeating {
                            used[pos] = true;
                            res.insert(layout[pos].name.clone(), itemix);
                        }
                        else {
                            let count = repcount.entry(layout[pos].name.clone())
                                .and_modify(|val| { *val += 1 })
                                .or_insert(1);
                            let tempname = format!("{}{}", layout[pos].name, count);
                            res.insert(tempname, itemix);
                        }
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

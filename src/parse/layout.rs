use std::fmt;
use std::collections::HashMap;
use lazy_static::lazy_static;

use crate::op::{Op1Def, Op3Def};
use crate::pixel::Pix;
use crate::param::Param;
use crate::parse::tree::{ParseTerm, ParseNode};
use crate::parse::{BuildOp1, BuildOp3};
use crate::parse::{parse_for_op1, parse_for_op3, parse_for_number, parse_for_color};

pub enum OpLayoutType {
    Op1,
    Op3,
    Number,
    Color,
    Param,
    Wave,
}

pub struct OpLayoutParam {
    pub name: String,
    pub ptype: OpLayoutType,
    pub optional: bool,
}

impl OpLayoutParam {
    fn param(name: &str, ptype: OpLayoutType) -> OpLayoutParam {
        OpLayoutParam {
            name: name.to_string(),
            ptype: ptype,
            optional: false,
        }
    }

    fn param_optional(name: &str, ptype: OpLayoutType) -> OpLayoutParam {
        OpLayoutParam {
            name: name.to_string(),
            ptype: ptype,
            optional: true,
        }
    }
}

type BuildFuncParam = fn(&ParseNode, &HashMap<String, usize>)->Result<Param, String>;
type BuildFuncOp1 = fn(&ParseNode, &HashMap<String, usize>)->Result<BuildOp1, String>;
type BuildFuncOp3 = fn(&ParseNode, &HashMap<String, usize>)->Result<BuildOp3, String>;

pub fn get_param_layout(val: &str) -> Option<&Vec<OpLayoutParam>> {
    return PARAMLAYOUT.get(val.to_lowercase().as_str());
}

pub fn get_op1_layout(val: &str) -> Option<&(Vec<OpLayoutParam>, BuildFuncOp1)> {
    return OP1LAYOUT.get(val.to_lowercase().as_str());
}

pub fn get_op3_layout(val: &str) -> Option<&(Vec<OpLayoutParam>, BuildFuncOp3)> {
    return OP3LAYOUT.get(val.to_lowercase().as_str());
}

lazy_static! {
    static ref PARAMLAYOUT: HashMap<&'static str, Vec<OpLayoutParam>> = {
        let mut map = HashMap::new();
        map.insert("constant", vec![
            OpLayoutParam::param("_1", OpLayoutType::Number),
        ]);
        map.insert("randflat", vec![
            OpLayoutParam::param("min", OpLayoutType::Number),
            OpLayoutParam::param("max", OpLayoutType::Number),
        ]);
        map.insert("randnorm", vec![
            OpLayoutParam::param_optional("mean", OpLayoutType::Number),
            OpLayoutParam::param_optional("stdev", OpLayoutType::Number),
        ]);
        map.insert("changing", vec![
            OpLayoutParam::param("start", OpLayoutType::Number),
            OpLayoutParam::param("velocity", OpLayoutType::Number),
        ]);
        map.insert("wave", vec![
            OpLayoutParam::param("shape", OpLayoutType::Wave),
            OpLayoutParam::param_optional("min", OpLayoutType::Number),
            OpLayoutParam::param_optional("max", OpLayoutType::Number),
            OpLayoutParam::param_optional("duration", OpLayoutType::Number),
        ]);
        map.insert("wavecycle", vec![
            OpLayoutParam::param("shape", OpLayoutType::Wave),
            OpLayoutParam::param_optional("min", OpLayoutType::Number),
            OpLayoutParam::param_optional("max", OpLayoutType::Number),
            OpLayoutParam::param_optional("period", OpLayoutType::Number),
        ]);
        map
    };

    static ref OP1LAYOUT: HashMap<&'static str, (Vec<OpLayoutParam>, BuildFuncOp1)> = {
        let mut map = HashMap::new();
        
        map.insert(
            "constant",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Number),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp1, String> {
                 let val = parse_for_number(&nod.params.items[pmap["_1"]])?;
                 let op = Op1Def::Constant(val);
                 Ok(BuildOp1::new(op))
             } as BuildFuncOp1)
        );
        
        map
    };
    
    static ref OP3LAYOUT: HashMap<&'static str, (Vec<OpLayoutParam>, BuildFuncOp3)> = {
        let mut map = HashMap::new();
        
        map.insert(
            "constant",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Color),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp3, String> {
                 let pix = parse_for_color(&nod.params.items[pmap["_1"]])?;
                 let op = Op3Def::Constant(pix);
                 Ok(BuildOp3::new(op))
             } as BuildFuncOp3)
        );
        
        map.insert(
            "invert",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op3),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp3, String> {
                 let subop = parse_for_op3(&nod.params.items[pmap["_1"]])?;
                 let op = Op3Def::Invert(0);
                 Ok(BuildOp3::new(op).addchild3(subop))
             } as BuildFuncOp3)
        );
        
        map.insert(
            "grey",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op1),
            ], |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp3, String> {
                let subop = parse_for_op1(&nod.params.items[pmap["_1"]])?;
                let op = Op3Def::Grey(0);
                Ok(BuildOp3::new(op).addchild1(subop))
            } as BuildFuncOp3)
        );
        
        map.insert(
            "rgb",
            (vec![
                OpLayoutParam::param("r", OpLayoutType::Op1),
                OpLayoutParam::param("g", OpLayoutType::Op1),
                OpLayoutParam::param("b", OpLayoutType::Op1),
            ], |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp3, String> {
                let subop1 = parse_for_op1(&nod.params.items[pmap["r"]])?;
                let subop2 = parse_for_op1(&nod.params.items[pmap["g"]])?;
                let subop3 = parse_for_op1(&nod.params.items[pmap["b"]])?;
                let op = Op3Def::RGB(0, 0, 0);
                Ok(BuildOp3::new(op).addchild1(subop1).addchild1(subop2).addchild1(subop3))
            } as BuildFuncOp3)
        );
        
        map
    };
}

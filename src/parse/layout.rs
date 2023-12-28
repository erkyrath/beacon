use std::fmt;
use std::collections::HashMap;
use lazy_static::lazy_static;

use crate::op::{Op1Def, Op3Def};
use crate::pixel::Pix;
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

type BuildFuncOp3 = fn(&ParseNode, &HashMap<String, usize>)->Result<BuildOp3, String>;

pub fn get_param_layout() -> &'static HashMap<&'static str, Vec<OpLayoutParam>> {
    return &PARAMLAYOUT;
}

pub fn get_op3_layout() -> &'static HashMap<&'static str, (Vec<OpLayoutParam>, BuildFuncOp3)> {
    return &OP3LAYOUT;
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
        
        map
    };
}

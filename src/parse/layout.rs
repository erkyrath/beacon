use std::fmt;
use std::collections::HashMap;
use lazy_static::lazy_static;

use crate::op::{Op1Def, Op3Def};
use crate::pixel::Pix;
use crate::waves::WaveShape;
use crate::pulser::Pulser;
use crate::param::Param;
use crate::parse::tree::{ParseTerm, ParseNode};
use crate::parse::{BuildOp1, BuildOp3};
use crate::parse::{parse_for_op1, parse_for_op3, parse_for_number, parse_for_color, parse_for_waveshape, parse_for_param};

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

pub fn get_waveshape(val: &str) -> Option<&WaveShape> {
    return WAVESHAPELAYOUT.get(val.to_lowercase().as_str());
}

pub fn get_param_layout(val: &str) -> Option<&(Vec<OpLayoutParam>, BuildFuncParam)> {
    return PARAMLAYOUT.get(val.to_lowercase().as_str());
}

pub fn get_op1_layout(val: &str) -> Option<&(Vec<OpLayoutParam>, BuildFuncOp1)> {
    return OP1LAYOUT.get(val.to_lowercase().as_str());
}

pub fn get_op3_layout(val: &str) -> Option<&(Vec<OpLayoutParam>, BuildFuncOp3)> {
    return OP3LAYOUT.get(val.to_lowercase().as_str());
}

lazy_static! {
    static ref WAVESHAPELAYOUT: HashMap<&'static str, WaveShape> = {
        HashMap::from([
            ("flat", WaveShape::Flat),
            ("square", WaveShape::Square),
            ("triangle", WaveShape::Triangle),
            ("sawtooth", WaveShape::SawTooth),
            ("sqrtooth", WaveShape::SqrTooth),
            ("sawdecay", WaveShape::SawDecay),
            ("sqrdecay", WaveShape::SqrDecay),
            ("sine", WaveShape::Sine),
        ])
    };
    
    static ref PARAMLAYOUT: HashMap<&'static str, (Vec<OpLayoutParam>, BuildFuncParam)> = {
        let mut map = HashMap::new();
        
        map.insert(
            "constant",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Number),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<Param, String> {
                 let val = parse_for_number(&nod.params.items[pmap["_1"]])?;
                 Ok(Param::Constant(val))
             } as BuildFuncParam)
        );

        map.insert(
            "randflat",
            (vec![
                OpLayoutParam::param("min", OpLayoutType::Number),
                OpLayoutParam::param("max", OpLayoutType::Number),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<Param, String> {
                 let min = parse_for_number(&nod.params.items[pmap["min"]])?;
                 let max = parse_for_number(&nod.params.items[pmap["max"]])?;
                 Ok(Param::RandFlat(min, max))
             } as BuildFuncParam)
        );
        
        map.insert(
            "randnorm",
            (vec![
                OpLayoutParam::param_optional("mean", OpLayoutType::Number),
                OpLayoutParam::param_optional("stdev", OpLayoutType::Number),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<Param, String> {
                 let mean = match pmap.get("mean") {
                     Some(val) => parse_for_number(&nod.params.items[*val])?,
                     None => 0.5,
                 };
                 let stdev = match pmap.get("stdev") {
                     Some(val) => parse_for_number(&nod.params.items[*val])?,
                     None => 0.25,
                 };
                 Ok(Param::RandNorm(mean, stdev))
             } as BuildFuncParam)
        );
        
        map.insert(
            "changing",
            (vec![
                OpLayoutParam::param("start", OpLayoutType::Number),
                OpLayoutParam::param("velocity", OpLayoutType::Number),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<Param, String> {
                 let start = parse_for_number(&nod.params.items[pmap["start"]])?;
                 let velocity = parse_for_number(&nod.params.items[pmap["velocity"]])?;
                 Ok(Param::Changing(start, velocity))
             } as BuildFuncParam)
        );
        
        map.insert(
            "wave",
            (vec![
                OpLayoutParam::param("shape", OpLayoutType::Wave),
                OpLayoutParam::param_optional("min", OpLayoutType::Number),
                OpLayoutParam::param_optional("max", OpLayoutType::Number),
                OpLayoutParam::param_optional("duration", OpLayoutType::Number),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<Param, String> {
                 let shape = parse_for_waveshape(&nod.params.items[pmap["shape"]])?;
                 let min = match pmap.get("min") {
                     Some(val) => parse_for_number(&nod.params.items[*val])?,
                     None => 0.0,
                 };
                 let max = match pmap.get("max") {
                     Some(val) => parse_for_number(&nod.params.items[*val])?,
                     None => 1.0,
                 };
                 let duration = match pmap.get("duration") {
                     Some(val) => parse_for_number(&nod.params.items[*val])?,
                     None => 1.0,
                 };
                 Ok(Param::Wave(shape, min, max, duration))
             } as BuildFuncParam)
        );
        
        map.insert(
            "wavecycle",
            (vec![
                OpLayoutParam::param("shape", OpLayoutType::Wave),
                OpLayoutParam::param_optional("min", OpLayoutType::Number),
                OpLayoutParam::param_optional("max", OpLayoutType::Number),
                OpLayoutParam::param_optional("period", OpLayoutType::Number),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<Param, String> {
                 let shape = parse_for_waveshape(&nod.params.items[pmap["shape"]])?;
                 let min = match pmap.get("min") {
                     Some(val) => parse_for_number(&nod.params.items[*val])?,
                     None => 0.0,
                 };
                 let max = match pmap.get("max") {
                     Some(val) => parse_for_number(&nod.params.items[*val])?,
                     None => 1.0,
                 };
                 let period = match pmap.get("period") {
                     Some(val) => parse_for_number(&nod.params.items[*val])?,
                     None => 1.0,
                 };
                 Ok(Param::WaveCycle(shape, min, max, period))
             } as BuildFuncParam)
        );
        
        map.insert(
            "quote",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Param),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<Param, String> {
                 let val = parse_for_param(&nod.params.items[pmap["_1"]])?;
                 Ok(Param::Quote(Box::new(val)))
             } as BuildFuncParam)
        );

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
        
        map.insert(
            "invert",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op1),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp1, String> {
                 let subop = parse_for_op1(&nod.params.items[pmap["_1"]])?;
                 let op = Op1Def::Invert();
                 Ok(BuildOp1::new(op).addchild1(subop))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "brightness",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op3),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp1, String> {
                 let subop = parse_for_op3(&nod.params.items[pmap["_1"]])?;
                 let op = Op1Def::Brightness();
                 Ok(BuildOp1::new(op).addchild3(subop))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "pulser",
            (vec![
                OpLayoutParam::param_optional("interval", OpLayoutType::Param),
                OpLayoutParam::param_optional("countlimit", OpLayoutType::Number),
                OpLayoutParam::param_optional("duration", OpLayoutType::Param),
                OpLayoutParam::param_optional("pos", OpLayoutType::Param),
                OpLayoutParam::param_optional("width", OpLayoutType::Param),
                OpLayoutParam::param_optional("spaceshape", OpLayoutType::Wave),
                OpLayoutParam::param_optional("timeshape", OpLayoutType::Wave),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp1, String> {
                 let mut pulser = Pulser::new();
                 if let Some(val) = pmap.get("interval") {
                     pulser.interval = parse_for_param(&nod.params.items[*val])?;
                 }
                 if let Some(val) = pmap.get("duration") {
                     pulser.duration = parse_for_param(&nod.params.items[*val])?;
                 }
                 if let Some(val) = pmap.get("pos") {
                     pulser.pos = parse_for_param(&nod.params.items[*val])?;
                 }
                 if let Some(val) = pmap.get("width") {
                     pulser.width = parse_for_param(&nod.params.items[*val])?;
                 }
                 if let Some(val) = pmap.get("countlimit") {
                     let limit = parse_for_number(&nod.params.items[*val])? as usize;
                     pulser.countlimit = Some(limit);
                 }
                 if let Some(val) = pmap.get("spaceshape") {
                     pulser.spaceshape = parse_for_waveshape(&nod.params.items[*val])?;
                 }
                 if let Some(val) = pmap.get("timeshape") {
                     pulser.timeshape = parse_for_waveshape(&nod.params.items[*val])?;
                 }
                 let op = Op1Def::Pulser(pulser);
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
                 let op = Op3Def::Invert();
                 Ok(BuildOp3::new(op).addchild3(subop))
             } as BuildFuncOp3)
        );
        
        map.insert(
            "grey",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op1),
            ], |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp3, String> {
                let subop = parse_for_op1(&nod.params.items[pmap["_1"]])?;
                let op = Op3Def::Grey();
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
                let op = Op3Def::RGB();
                Ok(BuildOp3::new(op).addchild1(subop1).addchild1(subop2).addchild1(subop3))
            } as BuildFuncOp3)
        );
        
        map.insert(
            "muls",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op3),
                OpLayoutParam::param("_2", OpLayoutType::Op1),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp3, String> {
                 let subop1 = parse_for_op3(&nod.params.items[pmap["_1"]])?;
                 let subop2 = parse_for_op1(&nod.params.items[pmap["_2"]])?;
                 let op = Op3Def::MulS();
                 Ok(BuildOp3::new(op).addchild3(subop1).addchild1(subop2))
             } as BuildFuncOp3)
        );
        
        map
    };
}

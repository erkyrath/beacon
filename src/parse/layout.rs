use std::fmt;
use std::collections::HashMap;
use lazy_static::lazy_static;

use crate::op::{Op1Def, Op3Def};
use crate::op::GradStop;
use crate::pixel::Pix;
use crate::waves::WaveShape;
use crate::pulser::Pulser;
use crate::param::{Param,ParamDef};
use crate::parse::tree::{ParseTerm, ParseNode};
use crate::parse::BuildOp;
use crate::parse::{parse_for_op1, parse_for_op3, parse_for_number, parse_for_color, parse_for_waveshape, parse_for_param, parse_for_gradstop};

pub enum OpLayoutType {
    Op1,
    Op3,
    Number,
    Color,
    Param,
    GradStop,
    Wave,
}

pub struct OpLayoutParam {
    pub name: String,
    pub ptype: OpLayoutType,
    pub optional: bool,
    pub repeating: bool,
}

impl OpLayoutParam {
    fn param(name: &str, ptype: OpLayoutType) -> OpLayoutParam {
        OpLayoutParam {
            name: name.to_string(),
            ptype: ptype,
            optional: false,
            repeating: false,
        }
    }

    fn param_optional(name: &str, ptype: OpLayoutType) -> OpLayoutParam {
        OpLayoutParam {
            name: name.to_string(),
            ptype: ptype,
            optional: true,
            repeating: false,
        }
    }
    
    fn param_repeating(name: &str, ptype: OpLayoutType) -> OpLayoutParam {
        OpLayoutParam {
            name: name.to_string(),
            ptype: ptype,
            optional: true,
            repeating: true,
        }
    }
}

type BuildFuncParam = fn(&ParseNode, &HashMap<String, usize>)->Result<Param, String>;
type BuildFuncGradStop = fn(&ParseNode, &HashMap<String, usize>)->Result<GradStop, String>;
type BuildFuncOp1 = fn(&ParseNode, &HashMap<String, usize>)->Result<BuildOp, String>;
type BuildFuncOp3 = fn(&ParseNode, &HashMap<String, usize>)->Result<BuildOp, String>;

pub fn get_waveshape(val: &str) -> Option<&WaveShape> {
    return WAVESHAPELAYOUT.get(val.to_lowercase().as_str());
}

pub fn get_gradstop_layout() -> &'static (Vec<OpLayoutParam>, BuildFuncGradStop) {
    return &GRADSTOPLAYOUT;
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
    static ref GRADSTOPLAYOUT: (Vec<OpLayoutParam>, BuildFuncGradStop) = {
        (vec![
            OpLayoutParam::param("pos", OpLayoutType::Number),
            OpLayoutParam::param("color", OpLayoutType::Color),
        ],
         |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<GradStop, String> {
             let pos = parse_for_number(&nod.params.items[pmap["pos"]])?;
             let color = parse_for_color(&nod.params.items[pmap["color"]])?;
             Ok(GradStop { pos:pos, color:color })
         } as BuildFuncGradStop)
    };
    
    static ref WAVESHAPELAYOUT: HashMap<&'static str, WaveShape> = {
        HashMap::from([
            ("flat", WaveShape::Flat),
            ("square", WaveShape::Square),
            ("halfsquare", WaveShape::HalfSquare),
            ("triangle", WaveShape::Triangle),
            ("trapezoid", WaveShape::Trapezoid),
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
                 Ok(Param::new(ParamDef::Constant(val)))
             } as BuildFuncParam)
        );

        map.insert(
            "randflat",
            (vec![
                OpLayoutParam::param("min", OpLayoutType::Number),
                OpLayoutParam::param("max", OpLayoutType::Number),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<Param, String> {
                 let min = parse_for_param(&nod.params.items[pmap["min"]])?;
                 let max = parse_for_param(&nod.params.items[pmap["max"]])?;
                 let pdef = ParamDef::RandFlat(0, 1);
                 Ok(Param::new(pdef).addchild(min).addchild(max))
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
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.5),
                 };
                 let stdev = match pmap.get("stdev") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.25),
                 };
                 let pdef = ParamDef::RandNorm(0, 1);
                 Ok(Param::new(pdef).addchild(mean).addchild(stdev))
             } as BuildFuncParam)
        );
        
        map.insert(
            "changing",
            (vec![
                OpLayoutParam::param("start", OpLayoutType::Number),
                OpLayoutParam::param("velocity", OpLayoutType::Number),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<Param, String> {
                 let start = parse_for_param(&nod.params.items[pmap["start"]])?;
                 let velocity = parse_for_param(&nod.params.items[pmap["velocity"]])?;
                 let pdef = ParamDef::Changing(0, 1);
                 Ok(Param::new(pdef).addchild(start).addchild(velocity))
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
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.0),
                 };
                 let max = match pmap.get("max") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(1.0),
                 };
                 let duration = match pmap.get("duration") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(1.0),
                 };
                 let pdef = ParamDef::Wave(shape, 0, 1, 2);
                 Ok(Param::new(pdef).addchild(min).addchild(max).addchild(duration))
             } as BuildFuncParam)
        );
        
        map.insert(
            "wavecycle",
            (vec![
                OpLayoutParam::param("shape", OpLayoutType::Wave),
                OpLayoutParam::param_optional("min", OpLayoutType::Number),
                OpLayoutParam::param_optional("max", OpLayoutType::Number),
                OpLayoutParam::param_optional("period", OpLayoutType::Number),
                OpLayoutParam::param_optional("offset", OpLayoutType::Number),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<Param, String> {
                 let shape = parse_for_waveshape(&nod.params.items[pmap["shape"]])?;
                 let min = match pmap.get("min") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.0),
                 };
                 let max = match pmap.get("max") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(1.0),
                 };
                 let period = match pmap.get("period") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(1.0),
                 };
                 let offset = match pmap.get("offset") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.0),
                 };
                 let pdef = ParamDef::WaveCycle(shape, 0, 1, 2, 3);
                 Ok(Param::new(pdef).addchild(min).addchild(max).addchild(period).addchild(offset))
             } as BuildFuncParam)
        );
        
        map.insert(
            "quote",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Param),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<Param, String> {
                 let val = parse_for_param(&nod.params.items[pmap["_1"]])?;
                 Ok(Param::new(ParamDef::Quote(Box::new(val))))
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
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let val = parse_for_number(&nod.params.items[pmap["_1"]])?;
                 let op = Op1Def::Constant(val);
                 Ok(BuildOp::new1(op))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "param",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Param),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let val = parse_for_param(&nod.params.items[pmap["_1"]])?;
                 let op = Op1Def::Param(val);
                 Ok(BuildOp::new1(op))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "wave",
            (vec![
                OpLayoutParam::param("shape", OpLayoutType::Wave),
                OpLayoutParam::param_optional("min", OpLayoutType::Param),
                OpLayoutParam::param_optional("max", OpLayoutType::Param),
                OpLayoutParam::param_optional("pos", OpLayoutType::Param),
                OpLayoutParam::param_optional("width", OpLayoutType::Param),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let shape = parse_for_waveshape(&nod.params.items[pmap["shape"]])?;
                 let min = match pmap.get("min") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.0),
                 };
                 let max = match pmap.get("max") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(1.0),
                 };
                 let pos = match pmap.get("pos") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.5),
                 };
                 let width = match pmap.get("width") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(1.0),
                 };
                 let op = Op1Def::Wave(shape, min, max, pos, width);
                 Ok(BuildOp::new1(op))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "wavecycle",
            (vec![
                OpLayoutParam::param("shape", OpLayoutType::Wave),
                OpLayoutParam::param_optional("min", OpLayoutType::Param),
                OpLayoutParam::param_optional("max", OpLayoutType::Param),
                OpLayoutParam::param_optional("pos", OpLayoutType::Param),
                OpLayoutParam::param_optional("period", OpLayoutType::Param),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let shape = parse_for_waveshape(&nod.params.items[pmap["shape"]])?;
                 let min = match pmap.get("min") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.0),
                 };
                 let max = match pmap.get("max") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(1.0),
                 };
                 let pos = match pmap.get("pos") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.5),
                 };
                 let period = match pmap.get("period") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(1.0),
                 };
                 let op = Op1Def::WaveCycle(shape, min, max, pos, period);
                 Ok(BuildOp::new1(op))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "invert",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op1),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let subop = parse_for_op1(&nod.params.items[pmap["_1"]])?;
                 let op = Op1Def::Invert();
                 Ok(BuildOp::new1(op).addchild1(subop))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "brightness",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op3),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let subop = parse_for_op3(&nod.params.items[pmap["_1"]])?;
                 let op = Op1Def::Brightness();
                 Ok(BuildOp::new1(op).addchild3(subop))
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
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
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
                 Ok(BuildOp::new1(op))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "decay",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op1),
                OpLayoutParam::param_optional("halflife", OpLayoutType::Param),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let subop = parse_for_op1(&nod.params.items[pmap["_1"]])?;
                 let halflife = match pmap.get("halflife") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(1.0),
                 };
                 let op = Op1Def::Decay(halflife);
                 Ok(BuildOp::new1(op).addchild1(subop))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "timedelta",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op1),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let subop = parse_for_op1(&nod.params.items[pmap["_1"]])?;
                 let op = Op1Def::TimeDelta();
                 Ok(BuildOp::new1(op).addchild1(subop))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "mul",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op1),
                OpLayoutParam::param("_2", OpLayoutType::Op1),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let subop1 = parse_for_op1(&nod.params.items[pmap["_1"]])?;
                 let subop2 = parse_for_op1(&nod.params.items[pmap["_2"]])?;
                 let op = Op1Def::Mul();
                 Ok(BuildOp::new1(op).addchild1(subop1).addchild1(subop2))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "sum",
            (vec![
                OpLayoutParam::param_repeating("_", OpLayoutType::Op1),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let op = Op1Def::Sum();
                 let mut bop = BuildOp::new1(op);
                 for ix in 0..pmap.len() {
                     let tempname = format!("_{}", 1+ix);
                     let subop = parse_for_op1(&nod.params.items[pmap[&tempname]])?;
                     bop = bop.addchild1(subop);
                 }
                 Ok(bop)
             } as BuildFuncOp1)
        );
        
        map.insert(
            "mean",
            (vec![
                OpLayoutParam::param_repeating("_", OpLayoutType::Op1),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let op = Op1Def::Mean();
                 let mut bop = BuildOp::new1(op);
                 for ix in 0..pmap.len() {
                     let tempname = format!("_{}", 1+ix);
                     let subop = parse_for_op1(&nod.params.items[pmap[&tempname]])?;
                     bop = bop.addchild1(subop);
                 }
                 Ok(bop)
             } as BuildFuncOp1)
        );
        
        map.insert(
            "min",
            (vec![
                OpLayoutParam::param_repeating("_", OpLayoutType::Op1),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let op = Op1Def::Min();
                 let mut bop = BuildOp::new1(op);
                 for ix in 0..pmap.len() {
                     let tempname = format!("_{}", 1+ix);
                     let subop = parse_for_op1(&nod.params.items[pmap[&tempname]])?;
                     bop = bop.addchild1(subop);
                 }
                 Ok(bop)
             } as BuildFuncOp1)
        );
        
        map.insert(
            "max",
            (vec![
                OpLayoutParam::param_repeating("_", OpLayoutType::Op1),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let op = Op1Def::Max();
                 let mut bop = BuildOp::new1(op);
                 for ix in 0..pmap.len() {
                     let tempname = format!("_{}", 1+ix);
                     let subop = parse_for_op1(&nod.params.items[pmap[&tempname]])?;
                     bop = bop.addchild1(subop);
                 }
                 Ok(bop)
             } as BuildFuncOp1)
        );
        
        map.insert(
            "clamp",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op1),
                OpLayoutParam::param_optional("min", OpLayoutType::Param),
                OpLayoutParam::param_optional("max", OpLayoutType::Param),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let subop = parse_for_op1(&nod.params.items[pmap["_1"]])?;
                 let min = match pmap.get("min") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.0),
                 };
                 let max = match pmap.get("max") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(1.0),
                 };
                 let op = Op1Def::Clamp(min, max);
                 Ok(BuildOp::new1(op).addchild1(subop))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "shift",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op1),
                OpLayoutParam::param_optional("offset", OpLayoutType::Param),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let subop = parse_for_op1(&nod.params.items[pmap["_1"]])?;
                 let offset = match pmap.get("offset") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.0),
                 };
                 let op = Op1Def::Shift(offset);
                 Ok(BuildOp::new1(op).addchild1(subop))
             } as BuildFuncOp1)
        );
        
        map.insert(
            "noise",
            (vec![
                OpLayoutParam::param_optional("grain", OpLayoutType::Number),
                OpLayoutParam::param_optional("octaves", OpLayoutType::Number),
                OpLayoutParam::param_optional("offset", OpLayoutType::Param),
                OpLayoutParam::param_optional("max", OpLayoutType::Param),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let grain = match pmap.get("grain") {
                     Some(val) => parse_for_number(&nod.params.items[*val])?,
                     None => 32.0,
                 };
                 let octaves = match pmap.get("octaves") {
                     Some(val) => parse_for_number(&nod.params.items[*val])?,
                     None => 1.0,
                 };
                 let max = match pmap.get("max") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(1.0),
                 };
                 let offset = match pmap.get("offset") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.0),
                 };
                 let op = Op1Def::Noise(grain as usize, octaves as usize, offset, max);
                 Ok(BuildOp::new1(op))
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
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let pix = parse_for_color(&nod.params.items[pmap["_1"]])?;
                 let op = Op3Def::Constant(pix);
                 Ok(BuildOp::new3(op))
             } as BuildFuncOp3)
        );
        
        map.insert(
            "invert",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op3),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let subop = parse_for_op3(&nod.params.items[pmap["_1"]])?;
                 let op = Op3Def::Invert();
                 Ok(BuildOp::new3(op).addchild3(subop))
             } as BuildFuncOp3)
        );
        
        map.insert(
            "grey",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op1),
            ], |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                let subop = parse_for_op1(&nod.params.items[pmap["_1"]])?;
                let op = Op3Def::Grey();
                Ok(BuildOp::new3(op).addchild1(subop))
            } as BuildFuncOp3)
        );
        
        map.insert(
            "rgb",
            (vec![
                OpLayoutParam::param("r", OpLayoutType::Op1),
                OpLayoutParam::param("g", OpLayoutType::Op1),
                OpLayoutParam::param("b", OpLayoutType::Op1),
            ], |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                let subop1 = parse_for_op1(&nod.params.items[pmap["r"]])?;
                let subop2 = parse_for_op1(&nod.params.items[pmap["g"]])?;
                let subop3 = parse_for_op1(&nod.params.items[pmap["b"]])?;
                let op = Op3Def::RGB();
                Ok(BuildOp::new3(op).addchild1(subop1).addchild1(subop2).addchild1(subop3))
            } as BuildFuncOp3)
        );
        
        map.insert(
            "hsv",
            (vec![
                OpLayoutParam::param("h", OpLayoutType::Op1),
                OpLayoutParam::param("s", OpLayoutType::Op1),
                OpLayoutParam::param("v", OpLayoutType::Op1),
            ], |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                let subop1 = parse_for_op1(&nod.params.items[pmap["h"]])?;
                let subop2 = parse_for_op1(&nod.params.items[pmap["s"]])?;
                let subop3 = parse_for_op1(&nod.params.items[pmap["v"]])?;
                let op = Op3Def::HSV();
                Ok(BuildOp::new3(op).addchild1(subop1).addchild1(subop2).addchild1(subop3))
            } as BuildFuncOp3)
        );
        
        map.insert(
            "gradient",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op1),
                OpLayoutParam::param_repeating("stop", OpLayoutType::Color),
            ], |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                let subop = parse_for_op1(&nod.params.items[pmap["_1"]])?;
                let mut stops: Vec<Pix<f32>> = Vec::new();
                let mut ix = 0;
                loop {
                    ix += 1;
                    let tempname = format!("stop{}", ix);
                    if let Some(val) = pmap.get(&tempname) {
                        let col = parse_for_color(&nod.params.items[*val])?;
                        stops.push(col);
                    }
                    else {
                        break;
                    }
                }
                let op = Op3Def::Gradient(stops);
                Ok(BuildOp::new3(op).addchild1(subop))
            } as BuildFuncOp3)
        );
        
        map.insert(
            "pgradient",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op1),
                OpLayoutParam::param_repeating("stop", OpLayoutType::GradStop),
            ], |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                let subop = parse_for_op1(&nod.params.items[pmap["_1"]])?;
                let mut stops: Vec<GradStop> = Vec::new();
                let mut ix = 0;
                loop {
                    ix += 1;
                    let tempname = format!("stop{}", ix);
                    if let Some(val) = pmap.get(&tempname) {
                        let stop = parse_for_gradstop(&nod.params.items[*val])?;
                        stops.push(stop);
                    }
                    else {
                        break;
                    }
                }
                stops.sort_unstable_by(|stop1, stop2| stop1.pos.partial_cmp(&stop2.pos).unwrap());
                let op = Op3Def::PGradient(stops);
                Ok(BuildOp::new3(op).addchild1(subop))
            } as BuildFuncOp3)
        );
        
        map.insert(
            "muls",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op3),
                OpLayoutParam::param("_2", OpLayoutType::Op1),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let subop1 = parse_for_op3(&nod.params.items[pmap["_1"]])?;
                 let subop2 = parse_for_op1(&nod.params.items[pmap["_2"]])?;
                 let op = Op3Def::MulS();
                 Ok(BuildOp::new3(op).addchild3(subop1).addchild1(subop2))
             } as BuildFuncOp3)
        );
        
        map.insert(
            "sum",
            (vec![
                OpLayoutParam::param_repeating("_", OpLayoutType::Op3),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let op = Op3Def::Sum();
                 let mut bop = BuildOp::new3(op);
                 for ix in 0..pmap.len() {
                     let tempname = format!("_{}", 1+ix);
                     let subop = parse_for_op3(&nod.params.items[pmap[&tempname]])?;
                     bop = bop.addchild3(subop);
                 }
                 Ok(bop)
             } as BuildFuncOp3)
        );
        
        map.insert(
            "mean",
            (vec![
                OpLayoutParam::param_repeating("_", OpLayoutType::Op3),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let op = Op3Def::Mean();
                 let mut bop = BuildOp::new3(op);
                 for ix in 0..pmap.len() {
                     let tempname = format!("_{}", 1+ix);
                     let subop = parse_for_op3(&nod.params.items[pmap[&tempname]])?;
                     bop = bop.addchild3(subop);
                 }
                 Ok(bop)
             } as BuildFuncOp3)
        );
        
        map.insert(
            "min",
            (vec![
                OpLayoutParam::param_repeating("_", OpLayoutType::Op3),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let op = Op3Def::Min();
                 let mut bop = BuildOp::new3(op);
                 for ix in 0..pmap.len() {
                     let tempname = format!("_{}", 1+ix);
                     let subop = parse_for_op3(&nod.params.items[pmap[&tempname]])?;
                     bop = bop.addchild3(subop);
                 }
                 Ok(bop)
             } as BuildFuncOp3)
        );
        
        map.insert(
            "max",
            (vec![
                OpLayoutParam::param_repeating("_", OpLayoutType::Op3),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let op = Op3Def::Max();
                 let mut bop = BuildOp::new3(op);
                 for ix in 0..pmap.len() {
                     let tempname = format!("_{}", 1+ix);
                     let subop = parse_for_op3(&nod.params.items[pmap[&tempname]])?;
                     bop = bop.addchild3(subop);
                 }
                 Ok(bop)
             } as BuildFuncOp3)
        );
        
        map.insert(
            "lerp",
            (vec![
                OpLayoutParam::param("mask", OpLayoutType::Op1),
                OpLayoutParam::param("_1", OpLayoutType::Op3),
                OpLayoutParam::param("_2", OpLayoutType::Op3),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let subop1 = parse_for_op3(&nod.params.items[pmap["_1"]])?;
                 let subop2 = parse_for_op3(&nod.params.items[pmap["_2"]])?;
                 let subopm = parse_for_op1(&nod.params.items[pmap["mask"]])?;
                 let op = Op3Def::Lerp();
                 Ok(BuildOp::new3(op).addchild3(subop1).addchild3(subop2).addchild1(subopm))
             } as BuildFuncOp3)
        );
        
        map.insert(
            "mask",
            (vec![
                OpLayoutParam::param("mask", OpLayoutType::Op1),
                OpLayoutParam::param("_1", OpLayoutType::Op3),
                OpLayoutParam::param("_2", OpLayoutType::Op3),
                OpLayoutParam::param_optional("threshold", OpLayoutType::Param),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let subop1 = parse_for_op3(&nod.params.items[pmap["_1"]])?;
                 let subop2 = parse_for_op3(&nod.params.items[pmap["_2"]])?;
                 let subopm = parse_for_op1(&nod.params.items[pmap["mask"]])?;
                 let threshold = match pmap.get("threshold") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.5),
                 };
                 let op = Op3Def::Mask(threshold);
                 Ok(BuildOp::new3(op).addchild3(subop1).addchild3(subop2).addchild1(subopm))
             } as BuildFuncOp3)
        );
        
        map.insert(
            "shift",
            (vec![
                OpLayoutParam::param("_1", OpLayoutType::Op3),
                OpLayoutParam::param_optional("offset", OpLayoutType::Param),
            ],
             |nod: &ParseNode, pmap: &HashMap<String, usize>| -> Result<BuildOp, String> {
                 let subop = parse_for_op3(&nod.params.items[pmap["_1"]])?;
                 let offset = match pmap.get("offset") {
                     Some(val) => parse_for_param(&nod.params.items[*val])?,
                     None => Param::newconst(0.0),
                 };
                 let op = Op3Def::Shift(offset);
                 Ok(BuildOp::new3(op).addchild3(subop))
             } as BuildFuncOp3)
        );
        
        map
    };
}

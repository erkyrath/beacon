pub mod tree;

use crate::op::{Op1Def, Op3Def};
use crate::pixel::Pix;
use crate::parse::tree::{ParseTerm, ParseNode};

//###?
enum TreeCtx {
    Op1,
    Op3,
    Number,
    Color,
    Param,
}

pub fn parse_script(filename: &str) -> Result<(), String> {
    let itemls = tree::parse_tree(filename)?;

    for item in &itemls.items {
        let op3 = parse_for_op3(item)?;
        println!("### got op3 {:?}", op3);
    }
    
    return Ok(());
}

fn parse_for_op3(nod: &ParseNode) -> Result<Op3Def, String> {
    match &nod.term {
        ParseTerm::Color(pix) => {
            //### verify no children
            Ok(Op3Def::Constant(pix.clone()))
        },
        ParseTerm::Number(val) => {
            //### verify no children
            //### Grey(Constant(val)) would be better
            Ok(Op3Def::Constant(Pix::new(*val, *val, *val)))
        },
        _ => Err(format!("unimplemented at line {}", nod.linenum)),
    }
}


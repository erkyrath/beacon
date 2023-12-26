use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

use crate::pixel::Pix;

#[derive(Debug, Clone)]
pub enum ParseTerm {
    Number(f32),
    Color(Pix<f32>),
    Ident(String),
}

pub struct ParseItems {
    items: Vec<ParseNode>,
}

pub struct ParseNode {
    key: Option<String>,
    term: ParseTerm,
    params: Box<ParseItems>,
    indent: Option<usize>,
    linenum: usize,
}

impl ParseItems {
    pub fn new() -> ParseItems {
        ParseItems {
            items: Vec::default(),
        }
    }

    pub fn append_at(&mut self, node: ParseNode, depth: usize) {
        let mut its = self;
        for _ in 0..depth {
            if let Some(subnod) = its.items.last_mut() {
                its = subnod.params.as_mut();
            }
            else {
                panic!("no child at depth {depth}");
            }
        }
        
        its.items.push(node);
    }

    pub fn append_at_indent(&mut self, nodes: &mut Vec<ParseNode>, indent: usize) -> Result<(), String> {
        if let Some(subnod) = self.items.last_mut() {
            if let Some(subindent) = subnod.indent {
                if indent > subindent {
                    return subnod.params.append_at_indent(nodes, indent);
                }
                if indent != subindent {
                    return Err("indentation mismatch".to_string());
                }
            }
        }
        self.items.append(nodes);
        return Ok(());
    }
    
    pub fn dump(&self, indent: usize) {
        for item in &self.items {
            item.dump(indent);
        }
    }
}

impl ParseNode {
    pub fn new(key: Option<&str>, term: ParseTerm, indent: Option<usize>, linenum: usize) -> ParseNode {
        ParseNode {
            key: key.map(|val| val.to_string()),
            term: term,
            params: Box::new(ParseItems::new()),
            indent: indent,
            linenum: linenum,
        }
    }

    pub fn dump(&self, indent: usize) {
        let indentstr: String = "  ".repeat(indent);
        //println!("{}### linenum {}, indent {}", indentstr, self.linenum, self.indent); //###
        match &self.key {
            None => println!("{}_={:?}", indentstr, self.term),
            Some(key) => println!("{}{}={:?}", indentstr, key, self.term),
        }
        self.params.dump(indent+1);
    }
}

fn labelterm(val: &str) -> Result<(Option<&str>, ParseTerm), String> {
    let (label, term) = val.split_once('=')
        .map_or_else(
            || (None, val.trim()),
            |(keyv, restv)| (Some(keyv.trim()), restv.trim()));

    if let Ok(float) = term.parse::<f32>() {
        return Ok((label, ParseTerm::Number(float)));
    }

    if term.starts_with('$') && term.len() == 4 {
        if let (Ok(rval), Ok(gval), Ok(bval)) = (
            u32::from_str_radix(&term[1..2], 16),
            u32::from_str_radix(&term[2..3], 16),
            u32::from_str_radix(&term[3..4], 16)
        ) {
            return Ok((label, ParseTerm::Color(Pix::new((rval as f32)/15.0, (gval as f32)/15.0, (bval as f32)/15.0))));
        }
    }
    
    if term.starts_with('$') && term.len() == 7 {
        if let (Ok(rval), Ok(gval), Ok(bval)) = (
            u32::from_str_radix(&term[1..3], 16),
            u32::from_str_radix(&term[3..5], 16),
            u32::from_str_radix(&term[5..7], 16)
        ) {
            return Ok((label, ParseTerm::Color(Pix::new((rval as f32)/255.0, (gval as f32)/255.0, (bval as f32)/255.0))));
        }
    }
    
    return Ok((label, ParseTerm::Ident(term.to_string())));
}

pub fn parse_tree(filename: &str) -> Result<ParseItems, String> {
    let file = File::open(filename)
        .map_err(|err| {
            format!("{}: {}", filename, err.to_string())
        })?;
    let lineiter = BufReader::new(file).lines();

    let mut scriptitems = ParseItems::new();

    let mut linenum = 0;
    for rline in lineiter {
        let line = rline.map_err(|err| {
            format!("{}: {}", filename, err.to_string())
        })?;
        linenum += 1;
        let line = line.trim_end().replace("\t", "    ");
        let origlen = line.len();
        let line = line.trim_start();
        let indent = origlen - line.len();
        if line.len() == 0 || line.starts_with('#') {
            continue;
        }
        //println!("### line: {indent} '{line}'");

        let mut lineterms = ParseItems::new();
        let mut depth = 0;
        let mut vindent = Some(indent);
        let mut ltail: &str = line;
        
        while ltail.len() > 0 {
            let mut term: &str;
            match ltail.find([',', ':']) {
                None => {
                    term = ltail;
                    ltail = "";
                    let (termkey, termval) = labelterm(term)?;
                    let nod = ParseNode::new(termkey, termval, vindent, linenum);
                    lineterms.append_at(nod, depth);
                },
                Some(pos) => {
                    (term, ltail) = ltail.split_at(pos);
                    term = term.trim();
                    if term.len() == 0 {
                        return Err(format!("empty term at line {linenum}"));
                    }
                    if ltail.starts_with(',') {
                        ltail = ltail.get(1..).unwrap().trim();
                        let (termkey, termval) = labelterm(term)?;
                        let nod = ParseNode::new(termkey, termval, vindent, linenum);
                        lineterms.append_at(nod, depth);
                    }
                    else {
                        ltail = ltail.get(1..).unwrap().trim();
                        let (termkey, termval) = labelterm(term)?;
                        let nod = ParseNode::new(termkey, termval, vindent, linenum);
                        lineterms.append_at(nod, depth);
                        depth += 1;
                        vindent = None;
                    }
                }
            }
        }
        
        scriptitems.append_at_indent(&mut lineterms.items, indent)
            .map_err(|msg| format!("{msg} at line {linenum}"))?;
    }

    println!("### tree:");
    scriptitems.dump(0); //###
    
    Ok(scriptitems)
}


use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

#[derive(Debug, Clone)]
enum ParseTerm {
    Number(f32),
    Ident(String),
}

struct ParseItems {
    items: Vec<ParseNode>,
}

struct ParseNode {
    key: Option<String>,
    term: ParseTerm,
    params: Box<ParseItems>,
    indent: usize,
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
            if indent > subnod.indent {
                return subnod.params.append_at_indent(nodes, indent);
            }
            if indent != subnod.indent && subnod.indent != usize::MAX {
                return Err("indentation mismatch".to_string());
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
    pub fn new(key: Option<&str>, term: ParseTerm, indent: usize, linenum: usize) -> ParseNode {
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

pub fn parse_script(filename: &str) -> Result<(), String> {
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
        let mut vindent = indent;
        let mut ltail: &str = line;
        
        while ltail.len() > 0 {
            let mut term: &str;
            match ltail.find([',', ':']) {
                None => {
                    term = ltail;
                    ltail = "";
                    let (termkey, termval) = labelterm(term);
                    lineterms.append_at(ParseNode::new(termkey, ParseTerm::Ident(termval.to_string()), vindent, linenum), depth);
                },
                Some(pos) => {
                    (term, ltail) = ltail.split_at(pos);
                    term = term.trim();
                    if term.len() == 0 {
                        return Err(format!("empty term at line {linenum}"));
                    }
                    if ltail.starts_with(',') {
                        ltail = ltail.get(1..).unwrap().trim();
                        lineterms.append_at(ParseNode::new(None, ParseTerm::Ident(term.to_string()), vindent, linenum), depth);
                    }
                    else {
                        ltail = ltail.get(1..).unwrap().trim();
                        lineterms.append_at(ParseNode::new(None, ParseTerm::Ident(term.to_string()), vindent, linenum), depth);
                        depth += 1;
                        vindent = usize::MAX;
                    }
                }
            }
        }
        
        scriptitems.append_at_indent(&mut lineterms.items, indent)
            .map_err(|msg| format!("{msg} at line {linenum}"))?;
    }

    //###
    scriptitems.dump(0);
    
    Ok(())
}

fn labelterm(val: &str) -> (Option<&str>, &str) {
    val.split_once('=')
        .map_or_else(
            || (None, val.trim()),
            |(keyv, restv)| (Some(keyv.trim()), restv.trim()))
}

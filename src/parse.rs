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
}

impl ParseItems {
    pub fn new() -> ParseItems {
        ParseItems {
            items: Vec::default(),
        }
    }

    pub fn append_at(&mut self, node: ParseNode, depth: usize) {
        if depth == 0 {
            self.items.push(node);
        }
        else {
            if let Some(subnod) = self.items.last_mut() {
                subnod.params.append_at(node, depth-1);
            }
            else {
                panic!("no child at depth");
            }
        }
    }

    pub fn dump(&self, indent: usize) {
        for item in &self.items {
            item.dump(indent);
        }
    }
}

impl ParseNode {
    pub fn new(key: Option<&str>, term: ParseTerm) -> ParseNode {
        ParseNode {
            key: key.map(|val| val.to_string()),
            term: term,
            params: Box::new(ParseItems::new()),
        }
    }

    pub fn dump(&self, indent: usize) {
        let indentstr: String = "  ".repeat(indent);
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

    let mut _linenum = 0;
    for rline in lineiter {
        let line = rline.map_err(|err| {
            format!("{}: {}", filename, err.to_string())
        })?;
        _linenum += 1;
        let line = line.trim_end().replace("\t", "    ");
        let origlen = line.len();
        let line = line.trim_start();
        let _indent = origlen - line.len();
        if line.len() == 0 || line.starts_with('#') {
            continue;
        }
        //println!("### {indent} '{line}'");

        let mut lineterms = ParseItems::new();
        let mut depth = 0;
        let mut ltail: &str = line;
        
        while ltail.len() > 0 {
            let mut term: &str;
            match ltail.find([',', ':']) {
                None => {
                    term = ltail;
                    ltail = "";
                    let (termkey, termval) = labelterm(term);
                    lineterms.append_at(ParseNode::new(termkey, ParseTerm::Ident(termval.to_string())), depth);
                },
                Some(pos) => {
                    (term, ltail) = ltail.split_at(pos);
                    term = term.trim();
                    if ltail.starts_with(',') {
                        ltail = ltail.get(1..).unwrap().trim();
                        lineterms.append_at(ParseNode::new(None, ParseTerm::Ident(term.to_string())), depth);
                    }
                    else {
                        ltail = ltail.get(1..).unwrap().trim();
                        depth += 1;
                        lineterms.append_at(ParseNode::new(None, ParseTerm::Ident(term.to_string())), depth);
                    }
                }
            }
        }
        
        scriptitems.items.append(&mut lineterms.items);
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

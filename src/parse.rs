use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

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

pub fn parse_script(filename: &str) -> Result<(), String> {
    let file = File::open(filename)
        .map_err(|err| {
            format!("{}: {}", filename, err.to_string())
        })?;
    let lineiter = BufReader::new(file).lines();

    /*###
    let mut items = ParseItems {
        items: Vec::default(),
    };
    ###*/

    let mut _linenum = 0;
    for rline in lineiter {
        let line = rline.map_err(|err| {
            format!("{}: {}", filename, err.to_string())
        })?;
        _linenum += 1;
        let line = line.trim_end().replace("\t", "    ");
        let origlen = line.len();
        let line = line.trim_start();
        let indent = origlen - line.len();
        if line.len() == 0 || line.starts_with('#') {
            continue;
        }
        //println!("### {indent} '{line}'");

        let key: Option<&str>;
        let rest: &str;
        (key, rest) = line.split_once('=')
            .map_or_else(
                || (None, line),
                |(keyv, restv)| (Some(keyv.trim()), restv.trim()));

        if let Some(key) = key {
            println!("### {indent} '{key}' = '{rest}'");
        }
        else {
            println!("### {indent} '{rest}'");
        }
    }
    
    Ok(())
}

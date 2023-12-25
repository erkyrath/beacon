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

    for rline in lineiter {
        let mut line = rline.map_err(|err| {
            format!("{}: {}", filename, err.to_string())
        })?;
        let line = line.trim_end();
        println!("### '{line}'");
    }
    
    Ok(())
}

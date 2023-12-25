
enum ParseTerm {
    Number(f32),
    Ident(String),
}

struct ParseItems {
    items: Vec<ParseNode>,
}

struct ParseNode {
    term: ParseTerm,
    params: Box<ParseItems>,
}


pub mod tree;

pub fn parse_script(filename: &str) -> Result<(), String> {
    let _ = tree::parse_tree(filename)?;
    return Ok(());
}

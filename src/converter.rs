use std::fs;

mod parser;

pub fn convert(path: &std::path::Path) {
    let file_content = fs::read_to_string(path).expect(&format!("Failed to open file: {:?}", path));

    println!("{}", file_content);
}
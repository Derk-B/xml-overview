use std::fs;

mod config;
mod lexer;
mod parser;

use lexer::lex_tokens;
use parser::parse;

use crate::converter::config::Config;

pub fn convert(path: &std::path::Path) {
    let file_content = fs::read_to_string(path).expect(&format!("Failed to open file: {:?}", path));

    let lex_result = lex_tokens(file_content, Config::new(false));

    if let Ok(tokens) = lex_result {
        for token in tokens {
            println!("{:?}", token);
        }

        // let parse_result = parse(tokens);
    }
}

use std::fs;

mod config;
mod lexer;
mod parser;

use lexer::lex_tokens;

use crate::converter::{config::Config, lexer::lex_graph};

pub fn convert(path: &std::path::Path) -> String {
    let file_content = fs::read_to_string(path).expect(&format!("Failed to open file: {:?}", path));

    let lex_result = lex_tokens(file_content, Config::new(false));

    let mut result = String::new();
    if let Ok(tokens) = lex_result {
        if let Ok(mut graph) = lex_graph(tokens) {
            graph.minimize();
            result = graph.print();
            // Remove tags for root level element that was not in the original xml file but is generated in this tool.
            result.replace_range(0..3, "");
            result.replace_range(result.len() - 5..result.len(), "");
        }
    }

    result
}

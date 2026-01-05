pub mod block;
mod errors;
pub mod token;

use core::panic;

use errors::LexError;
use token::Token;

use crate::converter::{config::Config, lexer::block::Graph};

type LexResult = (Token, String);

fn lex_tag_open(file: String, tag: &str) -> Option<(String, String)> {
    if !file.starts_with(tag) {
        return None;
    }

    let offset = tag.len();

    let mut file_remainder = file.clone();

    // If any of these lexers return a token, then we've reached the end of the tag name.
    let closing_lexers = [
        lex_tag_self_closing,
        lex_tag_closing,
        lex_comment,
        lex_whitespace,
        lex_newline,
    ];

    let mut str_body_len = 0;
    while file_remainder.len() > 0 {
        if closing_lexers
            .iter()
            .any(|lexer| lexer(file_remainder.clone()).is_some())
        {
            break;
        }

        str_body_len += 1;
        file_remainder = String::from(&file_remainder[1..]);
    }

    return Some((
        String::from(&file[offset..str_body_len]),
        String::from(&file[str_body_len..]),
    ));
}

fn lex_tag_open_start(file: String) -> Option<LexResult> {
    if let Some((name, remainder)) = lex_tag_open(file, "<") {
        return Some((Token::TagOpenStart(name), remainder));
    }

    None
}

fn lex_tag_close_start(file: String) -> Option<LexResult> {
    if let Some((name, remainder)) = lex_tag_open(file, "</") {
        return Some((Token::TagCloseStart(name), remainder));
    }

    None
}

fn lex_comment(file: String) -> Option<LexResult> {
    let comment_closing_tag = "-->";
    let comment_closing_tag_len = comment_closing_tag.len();

    let comment_opening_tag = "<!--";
    let comment_opening_tag_len = comment_opening_tag.len();

    if file.starts_with(comment_opening_tag) {
        let comment_end = file.find(comment_closing_tag);
        if let Some(index) = comment_end {
            return Some((
                Token::Comment(String::from(&file[comment_opening_tag_len..index])),
                String::from(&file[index + comment_closing_tag_len..]),
            ));
        }
    }

    None
}

fn lex_string(file: String) -> Option<LexResult> {
    let string_closing_tag = "\"";
    let offset = 1;
    if file.starts_with("\"") {
        let string_end = file[offset..].find(string_closing_tag);
        if let Some(end_pos) = string_end {
            let index = end_pos + offset;
            return Some((
                Token::String(String::from(&file[1..index])),
                String::from(&file[index + 1..]),
            ));
        }
    }

    None
}

fn lex_key(file: String) -> Option<LexResult> {
    let mut file_remainder = file.clone();

    // If any of these lexers return a token, then we've reached the end of this text token.
    let closing_lexers = [
        lex_tag_open_start,
        lex_tag_close_start,
        lex_tag_self_closing,
        lex_tag_closing,
        lex_comment,
        lex_string,
    ];

    let mut txt_body_len = 0;
    while file_remainder.len() > 0 {
        if closing_lexers
            .iter()
            .any(|lexer| lexer(file_remainder.clone()).is_some())
        {
            return None;
        }

        if file_remainder.starts_with('=') {
            break;
        }

        txt_body_len += 1;
        file_remainder = String::from(&file_remainder[1..]);
    }

    return Some((
        Token::Key(String::from(&file[0..txt_body_len])),
        String::from(&file[txt_body_len + 1..]),
    ));
}

fn lex_tag_self_closing(file: String) -> Option<LexResult> {
    if file.starts_with("/>") {
        return Some((Token::TagSelfClosing, String::from(&file[2..])));
    }

    None
}

fn lex_tag_closing(file: String) -> Option<LexResult> {
    if file.starts_with(">") {
        return Some((Token::TagClosing, String::from(&file[1..])));
    }

    None
}

fn lex_newline(file: String) -> Option<LexResult> {
    if file.starts_with('\n') {
        return Some((Token::Newline, String::from(&file[1..])));
    }

    None
}

fn lex_whitespace(file: String) -> Option<LexResult> {
    if [' ', '\t'].map(|c| Some(c)).contains(&file.chars().next()) {
        return Some((Token::Whitespace, String::from(&file[1..])));
    }

    None
}

fn lex_text(file: String) -> Option<LexResult> {
    let mut file_remainder = file.clone();

    // If any of these lexers return a token, then we've reached the end of this text token.
    let closing_lexers = [
        lex_tag_open_start,
        lex_tag_close_start,
        lex_tag_self_closing,
        lex_tag_closing,
        lex_comment,
    ];

    let mut txt_body_len = 0;
    while file_remainder.len() > 0 {
        if closing_lexers
            .iter()
            .any(|lexer| lexer(file_remainder.clone()).is_some())
        {
            break;
        }

        txt_body_len += 1;
        file_remainder = String::from(&file_remainder[1..]);
    }

    return Some((
        Token::Text(String::from(&file[0..txt_body_len])),
        String::from(&file[txt_body_len..]),
    ));
}

fn lex_token(file: String) -> Result<LexResult, LexError> {
    let lexers = [
        lex_comment,
        lex_string,
        lex_tag_close_start,
        lex_tag_open_start,
        lex_tag_self_closing,
        lex_tag_closing,
        lex_whitespace,
        lex_newline,
        lex_key,
        lex_text,
    ];

    for lexer in lexers {
        if let Some(r) = lexer(file.clone()) {
            return Ok(r);
        }
    }

    Err(LexError::UnexpectedString(file))
}

pub fn lex_tokens(file: String, config: Config) -> Result<Vec<Token>, LexError> {
    let mut file_to_lex = file;
    let mut tokens = Vec::<Token>::new();
    loop {
        let (token, file_remainder) = lex_token(file_to_lex.clone())?;
        tokens.push(token);

        if file_remainder.len() == 0 {
            break;
        }

        file_to_lex = file_remainder;
    }

    Ok(tokens)
}

pub fn lex_graph(tokens: Vec<Token>) -> Result<Graph, String> {
    let mut current_token = tokens.first().ok_or("No tokens available")?;
    let mut remaining_tokens = tokens[1..].to_vec();

    let mut graph: Graph = Graph::new();

    loop {
        match current_token {
            Token::TagOpenStart(tag_name) => {
                let closing_tag_pos = remaining_tokens
                    .iter()
                    .position(|t| *t == Token::TagClosing || *t == Token::TagSelfClosing)
                    .ok_or("Failed to find a closing tag")?;

                let keys_inside_tag = &remaining_tokens[1..closing_tag_pos]
                    .iter()
                    .filter(|t| match t {
                        Token::Key(_) => true,
                        _ => false,
                    })
                    .map(|t| match t {
                        Token::Key(key_name) => key_name.clone(),
                        _ => panic!("Fatal error: filter somehow failed for {:?}", t),
                    })
                    .collect::<Vec<String>>();

                let node_name = tag_name;
                let node_keys = keys_inside_tag;
                graph.add_node(node_name, node_keys);

                remaining_tokens = remaining_tokens[closing_tag_pos..].to_vec();
            }
            Token::TagCloseStart(_) => {
                let closing_tag_pos = remaining_tokens
                    .iter()
                    .position(|t| *t == Token::TagClosing)
                    .ok_or("Failed to find a closing tag")?;

                remaining_tokens = remaining_tokens[closing_tag_pos + 1..].to_vec();

                graph.close_current();
            }
            Token::TagClosing => {
                let next_opening_pos = remaining_tokens[1..].iter().position(|t| match t {
                    Token::TagCloseStart(_) => true,
                    Token::TagOpenStart(_) => true,
                    _ => false,
                });

                if let Some(pos) = next_opening_pos {
                    for token in remaining_tokens[1..pos].iter() {
                        graph.add_token(token);
                    }
                } else {
                    // End of file reached
                    break;
                }

                remaining_tokens = remaining_tokens[1..].to_vec();
            }
            Token::TagSelfClosing => {
                graph.close_current();
                remaining_tokens = remaining_tokens[1..].to_vec();
            }
            t => {
                graph.add_token(t);
                remaining_tokens = remaining_tokens[1..].to_vec();
            } // if let Some(block) =
        }

        if remaining_tokens.is_empty() {
            break;
        }

        current_token = &remaining_tokens[0];
    }
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use crate::converter::config::Config;

    use super::*;

    #[test]
    fn test_lex_next_token_open() {
        assert_eq!(
            lex_token(String::from("<element />")),
            Ok((
                Token::TagOpenStart(String::from("element")),
                String::from(" />")
            ))
        );

        assert_eq!(
            lex_token(String::from("<element/>")),
            Ok((
                Token::TagOpenStart(String::from("element")),
                String::from("/>")
            ))
        );

        assert_eq!(
            lex_token(String::from("</element />")),
            Ok((
                Token::TagCloseStart(String::from("element")),
                String::from(" />")
            ))
        );

        assert_eq!(
            lex_token(String::from("</element<!-- comment --> />")),
            Ok((
                Token::TagCloseStart(String::from("element")),
                String::from("<!-- comment --> />")
            ))
        );
    }

    #[test]
    fn test_lex_next_token_selfclosing() {
        assert_eq!(
            lex_token(String::from("/> ")),
            Ok((Token::TagSelfClosing, String::from(" ")))
        );
    }

    #[test]
    fn test_lex_next_token_close() {
        assert_eq!(
            lex_token(String::from("><")),
            Ok((Token::TagClosing, String::from("<")))
        );
    }

    #[test]
    fn test_lex_comment() {
        assert_eq!(
            lex_token(String::from("<!-- This is a comment -->")),
            Ok((
                Token::Comment(String::from(" This is a comment ")),
                String::from("")
            ))
        )
    }

    #[test]
    fn test_lex_next_token_string() {
        assert_eq!(
            lex_token(String::from("\"string content\" />")),
            Ok((
                Token::String(String::from("string content")),
                String::from(" />")
            ))
        );
    }

    #[test]
    fn test_lex_next_token_key() {
        assert_eq!(
            lex_token(String::from("<element />")),
            Ok((
                Token::TagOpenStart(String::from("element")),
                String::from(" />")
            ))
        );
    }

    #[test]
    fn test_lex_all_tokens() {
        assert_eq!(
            lex_tokens(String::from("</ <!-- comment --> >"), Config::new(false)),
            Ok(vec![
                Token::TagCloseStart(String::new()),
                Token::Whitespace,
                Token::Comment(String::from(" comment ")),
                Token::Whitespace,
                Token::TagClosing
            ])
        );
    }

    #[test]
    fn test_lex_all_tokens_from_file() {
        let file = String::from(
            "
        <tag>
            <child key=\"1\">Content</child>
            <child>More Content</child>
            <selfclosingchild/>
        </tag>
        ",
        );
        let result = lex_tokens(file, Config::new(false));

        assert!(result.is_ok());
    }
}

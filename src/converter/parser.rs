mod errors;
mod token;

use errors::LexError;
use token::Token;

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
    if let Some(c) = file.chars().next() {
        if !c.is_alphabetic() {
            return None;
        }
        let string_end = file[1..].find('=');
        if let Some(end_pos) = string_end {
            let index = end_pos + 1;
            return Some((
                Token::Key(String::from(&file[0..index])),
                String::from(&file[index + 1..]),
            ));
        }
    }

    None
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

fn lex_whitespace(file: String) -> Option<LexResult> {
    if [' ', '\t', '\n']
        .map(|c| Some(c))
        .contains(&file.chars().next())
    {
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

fn lex_tokens(file: String) -> Result<Vec<Token>, LexError> {
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

#[cfg(test)]
mod tests {
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
            lex_tokens(String::from("</ <!-- comment --> >")),
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
        let result = lex_tokens(file);

        assert!(result.is_ok());
    }
}

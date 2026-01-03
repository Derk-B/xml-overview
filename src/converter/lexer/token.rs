#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    TagOpenStart(String),
    TagCloseStart(String),
    TagSelfClosing,
    TagClosing,
    Key(String),
    String(String),
    Comment(String),
    Whitespace,
    Newline,
    Text(String), // Different from a String in the sence that a String is surrounded by double qoutes and Text is not.
}

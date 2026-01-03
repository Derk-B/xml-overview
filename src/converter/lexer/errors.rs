#[derive(Debug, PartialEq, Eq)]
pub enum LexError {
    UnexpectedString(String),
}

use crate::converter::lexer::token::Token;

#[derive(Debug, PartialEq, Eq)]
pub enum Block {
    BlockOpen(String, Vec<Token>), // Open xml block with block name and key names
    BlockClosing,                  // Closing xml block that closes a block pair
    BlockSeflClosing(String, Vec<Token>), // Selfclosing xml block with block name and key names
    Body(Vec<Token>),
}

pub enum BlocBodyItem {
    B(BlocOpen),
    T(Token),
}

pub trait Bloc {}
pub struct BlocOpen {
    pub parent: Option<Box<BlocOpen>>,
    pub keys: Vec<String>,
    pub body: Vec<BlocBodyItem>,
    pub name: String,
}
impl Bloc for BlocOpen {}

pub struct BlocClosing {}
impl Bloc for BlocClosing {}

pub struct BlocSelfClosing {}
impl Bloc for BlocSelfClosing {}

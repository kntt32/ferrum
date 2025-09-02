pub struct Tokenizer<'a> {
    s: &'a str,
    index: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(s: &'a str) -> Self {
        Self { s, index: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Token<'a> {
    Ident(&'a str),
    Function(&'a str),
    AtKeyword(&'a str),
    Hash(&'a str),
    String(&'a str),
    BadString,
    Url(&'a str),
    BadUrl,
    Delim(char),
    Number(Num),
    Percentage(Num),
    Dimension(Num, &'a str),
    Whitespace,
    Cdo,
    Cdc,
    Colon,
    Semicolon,
    Comma,
    LSquare,
    RSquare,
    LParen,
    RParen,
    LCurly,
    RCurly,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Num {
    Integer(i64),
    Floating(f64),
}

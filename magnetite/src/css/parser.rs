use super::tokenizer::*;

#[derive(Clone, Debug)]
pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(tokenizer: Tokenizer<'a>) -> Self {
        Self { tokenizer }
    }

    fn parse_a_style_sheet() {
        todo!()
    }
}

use super::tokenizer::*;

#[derive(Clone, Debug)]
pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    remaining_token: Option<Token>,
    top_level_flag: bool,
    stylesheet: Option<StyleSheet>,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(tokenizer: Tokenizer<'a>) -> Self {
        Self {
            tokenizer,
            remaining_token: None,
            top_level_flag: true,
            stylesheet: None,
            errors: Vec::new(),
        }
    }

    fn stylesheet(&mut self) -> &mut StyleSheet {
        self.stylesheet.as_mut().unwrap()
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    fn error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    fn consume(&mut self) -> Option<Token> {
        if let Some(token) = self.remaining_token.take() {
            Some(token)
        } else {
            self.tokenizer.step()
        }
    }

    fn reconsume(&mut self, token: Token) {
        assert!(self.remaining_token.is_none());
        self.remaining_token = Some(token);
    }

    pub fn parse_a_style_sheet(&mut self) -> StyleSheet {
        self.stylesheet = Some(StyleSheet::new());
        self.top_level_flag = true;
        self.consume_a_list_of_rules();
        self.stylesheet.take().unwrap()
    }

    fn consume_a_list_of_rules(&mut self) {
        while let Some(token) = self.consume() {
            match token {
                Token::Whitespace => {}
                Token::Cdo | Token::Cdc => {
                    if !self.top_level_flag {
                        self.reconsume(token);
                        if let Some(rule) = self.consume_a_qualified_rule() {
                            self.stylesheet().rules.push(Rule::StyleRule(rule));
                        }
                    }
                }
                Token::AtKeyword(..) => {
                    self.reconsume(token);
                    self.consume_an_at_rule();
                    if let Some(rule) = self.consume_an_at_rule() {
                        self.stylesheet().rules.push(Rule::AtRule(rule));
                    }
                }
                _ => {
                    self.reconsume(token);
                    if let Some(rule) = self.consume_a_qualified_rule() {
                        self.stylesheet().rules.push(Rule::StyleRule(rule));
                    }
                }
            }
        }
    }

    fn consume_an_at_rule(&mut self) -> Option<AtRule> {
        todo!()
    }

    fn consume_a_qualified_rule(&mut self) -> Option<StyleRule> {
        let mut style_rule = StyleRule::new();
        while let Some(token) = self.consume() {
            match token {
                Token::LCurly => {
                    style_rule.block = self.consume_a_simple_block(Token::RCurly);
                    style_rule.block.retain(|t| t != &Token::Whitespace);
                    return Some(style_rule);
                }
                _ => {
                    self.reconsume(token);
                    let mut component = self.consume_a_component_value();
                    style_rule.prelude.append(&mut component);
                }
            }
        }

        self.error(ParseError::UnexpectedEof);
        None
    }

    fn consume_a_simple_block(&mut self, ending_token: Token) -> Vec<Token> {
        let mut simple_block = Vec::new();

        while let Some(token) = self.consume() {
            if token == ending_token {
                return simple_block;
            }
            self.reconsume(token);
            let mut component = self.consume_a_component_value();
            simple_block.append(&mut component);
        }

        self.error(ParseError::UnexpectedEof);
        simple_block
    }

    fn consume_a_component_value(&mut self) -> Vec<Token> {
        let Some(token) = self.consume() else {
            return Vec::new();
        };

        match token {
            Token::LCurly => self.consume_a_simple_block(Token::RCurly),
            Token::LSquare => self.consume_a_simple_block(Token::RSquare),
            Token::LParen => self.consume_a_simple_block(Token::RParen),
            Token::Function(..) => self.consume_a_function(),
            _ => Vec::from([token]),
        }
    }

    fn consume_a_function(&mut self) -> Vec<Token> {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleSheet {
    location: Option<String>,
    rules: Vec<Rule>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self {
            location: None,
            rules: Vec::new(),
        }
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Rule {
    AtRule(AtRule),
    StyleRule(StyleRule),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtRule {}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleRule {
    prelude: Vec<Token>,
    block: Vec<Token>,
}

impl StyleRule {
    pub fn new() -> Self {
        Self {
            prelude: Vec::new(),
            block: Vec::new(),
        }
    }

    pub fn prelude(&self) -> &[Token] {
        &self.prelude
    }

    pub fn block(&self) -> &[Token] {
        &self.block
    }
}

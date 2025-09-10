use super::Num;
use super::Rule;
use super::StyleRule;
use super::StyleSheet;
use super::Token;
use crate::arena::Arena;
use crate::arena::NodeId;
use crate::render::Color;
use std::slice::Iter;

#[derive(Clone, Debug)]
pub struct CssomArena {
    roots: Vec<NodeId>,
    arena: Arena<CssomRule>,
}

impl CssomArena {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            arena: Arena::new(),
        }
    }

    pub fn add_stylesheet(&mut self, stylesheet: &StyleSheet) {
        let rules: &[Rule] = stylesheet.rules();

        for rule in rules {
            match rule {
                Rule::AtRule(..) => unimplemented!(),
                Rule::StyleRule(stylerule) => {
                    if let Some(id) = self.push_stylerule(stylerule) {
                        self.roots.push(id);
                    }
                }
            }
        }
    }

    fn push_stylerule(&mut self, stylerule: &StyleRule) -> Option<NodeId> {
        let cssom_stylerule = CssomStyleRule::from_stylerule(stylerule)?;
        let id = self.arena.push(CssomRule::StyleRule(cssom_stylerule));
        Some(id)
    }
}

#[derive(Clone, Debug)]
pub enum CssomRule {
    StyleRule(CssomStyleRule),
    AtRule(CssomAtRule),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssomStyleRule {
    selectors: Vec<Vec<Selector>>,
    style: CssomStyle,
}

impl CssomStyleRule {
    pub fn from_stylerule(stylerule: &StyleRule) -> Option<Self> {
        let selectors = Selector::from_tokens(stylerule.prelude())?;
        let style = CssomStyle::from_tokens(stylerule.block())?;
        Some(Self { selectors, style })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Selector {
    Type(String),
    Id(String),
    Class(String),
    Universal,
}

impl Selector {
    pub fn from_tokens(tokens: &[Token]) -> Option<Vec<Vec<Selector>>> {
        let mut selectors_list = Vec::new();
        let mut selectors = Vec::new();
        let mut prelude = tokens.iter();

        while let Some(token) = prelude.next() {
            match token {
                Token::Whitespace => {}
                Token::Ident(name) => {
                    selectors.push(Selector::Type(name.clone()));
                }
                Token::Hash { value, .. } => {
                    selectors.push(Selector::Id(value.clone()));
                }
                Token::Delim(c) if *c == '.' => {
                    let Some(Token::Ident(name)) = prelude.next() else {
                        return None;
                    };
                    selectors.push(Selector::Class(name.clone()));
                }
                Token::Delim(c) if *c == ',' => {
                    selectors_list.push(selectors);
                    selectors = Vec::new();
                }
                _ => todo!(),
            }
        }
        selectors_list.push(selectors);
        Some(selectors_list)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssomStyle {
    color: Option<Color>,
    font_size: Option<Value>,
}

impl CssomStyle {
    pub fn new() -> Self {
        Self {
            color: None,
            font_size: None,
        }
    }

    pub fn from_tokens(tokens: &[Token]) -> Option<Self> {
        let mut this = Self::new();
        let mut block = tokens.iter();

        while let Some(token) = block.next() {
            let Token::Ident(name) = token else {
                return None;
            };
            if block.next()? != &Token::Colon {
                return None;
            }

            let iter = block.clone().take_while(|t| *t != &Token::Semicolon);
            while let Some(token) = block.next()
                && token != &Token::Semicolon
            {}

            match name.as_str() {
                "color" => {
                    this.parse_color(iter);
                }
                "font-size" => {
                    this.parse_font_size(iter);
                }
                _ => continue,
            }
        }

        Some(this)
    }

    fn parse_font_size<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        match iter.next() {
            Some(Token::Dimension { value, unit }) => {
                if let Some(value) = Value::from(*value, unit) {
                    self.font_size = Some(value);
                }
            }
            _ => {}
        }
    }

    fn parse_color<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        match iter.next() {
            Some(Token::Ident(name)) => {
                if let Ok(color) = Color::from_name(name) {
                    self.color = Some(color);
                }
            }
            Some(Token::Hash { value, .. }) => {
                if let Ok(color) = Color::from_str_noprefix(value) {
                    self.color = Some(color);
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Pixel(Num),
}

impl Value {
    pub fn from(num: Num, unit: &str) -> Option<Self> {
        match unit {
            "px" => Some(Self::Pixel(num)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssomAtRule {}

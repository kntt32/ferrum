use super::Num;
use super::Rule;
use super::StyleRule;
use super::StyleSheet;
use super::Token;
use crate::arena::Arena;
use crate::arena::NodeId;
use crate::render::Color;
use crate::render::RenderArena;
use crate::render::RenderNodeType;
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

    pub fn search_style_rule<'a>(
        &'a self,
        render_arena: &RenderArena,
        id: NodeId,
    ) -> Option<CssomStyle> {
        for i in &self.roots {
            if let CssomRule::StyleRule(ref rule) = *self.arena[*i] {
                let selectors = &rule.selectors;
                for s in selectors {
                    if Selector::match_selectors_with(s, render_arena, id) {
                        return Some(rule.style.clone());
                    }
                }
            }
        }
        None
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
    pub fn match_with(&self, nodetype: &RenderNodeType) -> bool {
        match self {
            Self::Type(t) => {
                matches!(nodetype, RenderNodeType::Element{name, ..} if name == t)
            }
            Self::Universal => true,
            _ => panic!(),
        }
    }

    pub fn match_selectors_with(
        selectors: &[Self],
        render_arena: &RenderArena,
        mut id: NodeId,
    ) -> bool {
        let mut iter = selectors.iter().rev();
        if let Some(selector) = iter.next()
            && selector.match_with(render_arena[id].node_type())
        {
            let Some(i) = render_arena[id].parent() else {
                return false;
            };
            id = i;
            while let Some(selector) = iter.next() {
                if selector.match_with(render_arena[id].node_type()) {
                    let Some(i) = render_arena[id].parent() else {
                        return false;
                    };
                    id = i;
                }
            }
            true
        } else {
            false
        }
    }

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
    pub background_color: Option<Color>,
    pub color: Option<Color>,
    pub font_size: Option<Value>,
}

impl CssomStyle {
    pub fn new() -> Self {
        Self {
            background_color: None,
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
                "background-color" => {
                    this.parse_background_color(iter);
                }
                "background" => {
                    this.parse_background(iter);
                }
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

    fn parse_background_color<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        self.background_color = self.parse_part_color(&mut iter).or(self.background_color);
    }

    fn parse_background<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        self.background_color = self.parse_part_color(&mut iter).or(self.background_color);
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

    fn parse_part_color<'a>(
        &mut self,
        iter: &mut impl Iterator<Item = &'a Token>,
    ) -> Option<Color> {
        match iter.next() {
            Some(Token::Ident(name)) => Color::from_name(name).ok(),
            Some(Token::Hash { value, .. }) => Color::from_str_noprefix(value).ok(),
            _ => None,
        }
    }

    fn parse_color<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        self.color = self.parse_part_color(&mut iter).or(self.color);
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

    pub fn as_pixel(&self) -> f32 {
        match self {
            Self::Pixel(num) => num.as_floating() as f32,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssomAtRule {}

use super::CascadeOrd;
use super::Num;
use super::Origin;
use super::Parser;
use super::Rule;
use super::StyleRule;
use super::StyleSheet;
use super::Token;
use super::Tokenizer;
use crate::arena::Arena;
use crate::arena::NodeId;
use crate::render::Color;
use crate::render::RenderArena;
use crate::render::RenderNodeType;
use std::sync::LazyLock;

#[derive(Clone, Debug)]
pub struct CssomArena {
    roots: Vec<NodeId>,
    arena: Arena<CssomStyleRule>,
}

impl CssomArena {
    pub fn new() -> Self {
        let mut this = Self {
            roots: Vec::new(),
            arena: Arena::new(),
        };

        static USERAGENT_STYLESHEET: LazyLock<StyleSheet> = LazyLock::new(|| {
            let tokenizer = Tokenizer::new(include_str!("../../../html.css"));
            let parser = Parser::new(tokenizer);
            parser.parse_a_style_sheet()
        });

        this.add_stylesheet(&USERAGENT_STYLESHEET, Origin::UserAgent);
        this
    }

    pub fn add_stylesheet(&mut self, stylesheet: &StyleSheet, origin: Origin) {
        let rules: &[Rule] = stylesheet.rules();

        for rule in rules {
            match rule {
                Rule::AtRule(..) => {
                    println!("WARNING: AtRule was ignored at cssom.rs");
                }
                Rule::StyleRule(stylerule) => {
                    for s in CssomStyleRule::from_stylerule(stylerule, origin, false) {
                        let id = self.arena.push(s);
                        self.roots.push(id);
                    }
                }
            }
        }

        self.roots.sort_by(|lhs, rhs| {
            self.arena[*lhs]
                .selector
                .ord
                .cmp(&self.arena[*rhs].selector.ord)
        });
    }

    pub fn attach_style_for(&self, render_arena: &mut RenderArena, id: NodeId) {
        for i in &self.roots {
            let style = &self.arena[*i].style;
            let selector = &self.arena[*i].selector;
            if selector.match_with(render_arena, id) {
                render_arena[id].style.attach_cssom_style(style);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssomStyleRule {
    selector: Selector,
    style: CssomStyle,
}

impl CssomStyleRule {
    pub fn from_stylerule(stylerule: &StyleRule, origin: Origin, important: bool) -> Vec<Self> {
        if let Some(selectors) = Selector::from_tokens(stylerule.prelude(), origin, important)
            && let Some(style) = CssomStyle::from_tokens(stylerule.block())
        {
            let mut vec = Vec::new();
            for selector in selectors {
                vec.push(Self {
                    selector,
                    style: style.clone(),
                });
            }
            vec
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Selector {
    units: Vec<SelectorUnit>,
    ord: CascadeOrd,
}

impl Selector {
    pub fn new(origin: Origin, important: bool) -> Self {
        Self {
            units: Vec::new(),
            ord: CascadeOrd::new(origin, important),
        }
    }

    pub fn from_tokens(tokens: &[Token], origin: Origin, important: bool) -> Option<Vec<Self>> {
        let mut selectors = Vec::new();
        let mut this = Self::new(origin, important);
        let mut prelude = tokens.iter();

        while let Some(token) = prelude.next() {
            match token {
                Token::Whitespace => {}
                Token::Ident(name) => {
                    this.units.push(SelectorUnit::Type(name.clone()));
                }
                Token::Hash { value, .. } => {
                    this.units.push(SelectorUnit::Id(value.clone()));
                }
                Token::Delim(c) if *c == '.' => {
                    let Some(Token::Ident(name)) = prelude.next() else {
                        return None;
                    };
                    this.units.push(SelectorUnit::Class(name.clone()));
                }
                Token::Delim(c) if *c == ',' => {
                    selectors.push(this);
                    this = Self::new(origin, important);
                }
                _ => {
                    println!("WARNING: unimplemented in Selector::from_token");
                    return None;
                }
            }
        }

        selectors.push(this);
        Some(selectors)
    }

    pub fn match_with(&self, render_arena: &RenderArena, mut id: NodeId) -> bool {
        let mut iter = self.units.iter().rev();
        if let Some(unit) = iter.next()
            && unit.match_with(render_arena[id].node_type())
        {
            while let Some(unit) = iter.next() {
                let Some(i) = render_arena[id].parent() else {
                    return false;
                };
                id = i;
                if unit.match_with(render_arena[id].node_type()) {
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum SelectorUnit {
    Type(String),
    Id(String),
    Class(String),
    Universal,
}

impl SelectorUnit {
    pub fn match_with(&self, nodetype: &RenderNodeType) -> bool {
        match self {
            Self::Type(t) => {
                matches!(nodetype, RenderNodeType::Element{name, ..} if name == t)
            }
            Self::Universal => true,
            _ => panic!(),
        }
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

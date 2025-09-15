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
use crate::render::RenderStyle;
use std::ops::Deref;
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
                    // TODO
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

    pub fn rules(&self) -> &[NodeId] {
        &self.roots
    }
}

impl Deref for CssomArena {
    type Target = Arena<CssomStyleRule>;

    fn deref(&self) -> &Self::Target {
        &self.arena
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssomStyleRule {
    selector: Selector,
    style: CssomStyle,
}

impl CssomStyleRule {
    pub fn selector(&self) -> &Selector {
        &self.selector
    }

    pub fn style(&self) -> &CssomStyle {
        &self.style
    }

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
                    // TODO
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
    pub margin_top: Option<Value>,
    pub margin_right: Option<Value>,
    pub margin_bottom: Option<Value>,
    pub margin_left: Option<Value>,
    pub width: Option<Value>,
    pub height: Option<Value>,
}

impl CssomStyle {
    pub fn new() -> Self {
        Self {
            background_color: None,
            color: None,
            font_size: None,
            margin_top: None,
            margin_right: None,
            margin_bottom: None,
            margin_left: None,
            width: None,
            height: None,
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
                "margin-top" => {
                    this.parse_margin_top(iter);
                }
                "margin-right" => {
                    this.parse_margin_right(iter);
                }
                "margin-bottom" => {
                    this.parse_margin_bottom(iter);
                }
                "margin-left" => {
                    this.parse_margin_left(iter);
                }
                "margin" => {
                    this.parse_margin(iter);
                }
                "width" => {
                    this.parse_width(iter);
                }
                "height" => {
                    this.parse_height(iter);
                }
                _ => continue,
            }
        }

        Some(this)
    }

    fn parse_width<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        if let Some(width) = Self::parse_part_value(&mut iter) {
            self.width = Some(width);
        }
    }

    fn parse_height<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        if let Some(height) = Self::parse_part_value(&mut iter) {
            self.height = Some(height);
        }
    }

    fn parse_margin<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        let mut values = Vec::new();
        for _ in 0..4 {
            if let Some(value) = Self::parse_part_value(&mut iter) {
                values.push(value);
            }
        }

        match values.len() {
            0 => {}
            1 => {
                self.margin_top = Some(values[0].clone());
                self.margin_right = Some(values[0].clone());
                self.margin_bottom = Some(values[0].clone());
                self.margin_left = Some(values[0].clone());
            }
            2 => {
                self.margin_top = Some(values[0].clone());
                self.margin_right = Some(values[1].clone());
                self.margin_bottom = Some(values[0].clone());
                self.margin_left = Some(values[1].clone());
            }
            3 => {
                self.margin_top = Some(values[0].clone());
                self.margin_right = Some(values[1].clone());
                self.margin_bottom = Some(values[2].clone());
                self.margin_left = Some(values[1].clone());
            }
            4 => {
                self.margin_top = Some(values[0].clone());
                self.margin_right = Some(values[1].clone());
                self.margin_bottom = Some(values[2].clone());
                self.margin_left = Some(values[3].clone());
            }
            _ => unreachable!(),
        }
    }

    fn parse_margin_right<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        if let Some(margin_right) = Self::parse_part_value(&mut iter) {
            self.margin_right = Some(margin_right);
        }
    }

    fn parse_margin_bottom<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        if let Some(margin_bottom) = Self::parse_part_value(&mut iter) {
            self.margin_bottom = Some(margin_bottom);
        }
    }

    fn parse_margin_left<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        if let Some(margin_left) = Self::parse_part_value(&mut iter) {
            self.margin_left = Some(margin_left);
        }
    }

    fn parse_margin_top<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        if let Some(margin_top) = Self::parse_part_value(&mut iter) {
            self.margin_top = Some(margin_top);
        }
    }

    fn parse_background_color<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        self.background_color = Self::parse_part_color(&mut iter).or(self.background_color);
    }

    fn parse_background<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        self.background_color = Self::parse_part_color(&mut iter).or(self.background_color);
    }

    fn parse_font_size<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        if let Some(font_size) = Self::parse_part_value(&mut iter) {
            self.font_size = Some(font_size);
        }
    }

    fn parse_part_value<'a>(iter: &mut impl Iterator<Item = &'a Token>) -> Option<Value> {
        match iter.next() {
            Some(Token::Dimension { value, unit }) => Value::from(*value, unit),
            Some(Token::Ident(ident)) if ident.as_str() == "auto" => Some(Value::Auto),
            _ => None,
        }
    }

    fn parse_part_color<'a>(iter: &mut impl Iterator<Item = &'a Token>) -> Option<Color> {
        match iter.next() {
            Some(Token::Ident(name)) => Color::from_name(name).ok(),
            Some(Token::Hash { value, .. }) => Color::from_str_noprefix(value).ok(),
            _ => None,
        }
    }

    fn parse_color<'a>(&mut self, mut iter: impl Iterator<Item = &'a Token>) {
        self.color = Self::parse_part_color(&mut iter).or(self.color);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Pixel(Num),
    Em(Num),
    Auto,
}

impl Value {
    pub fn from(num: Num, unit: &str) -> Option<Self> {
        match unit {
            "px" => Some(Self::Pixel(num)),
            "em" => Some(Self::Em(num)),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }

    pub fn as_pixel(&self, render_arena: &RenderArena, id: NodeId) -> f32 {
        match self {
            Self::Pixel(num) => num.as_floating() as f32,
            Self::Em(num) => {
                let parent_size = if let Some(parent_id) = render_arena[id].parent() {
                    render_arena[parent_id].style().font_size
                } else {
                    16.0
                };
                parent_size * num.as_floating() as f32
            }
            Self::Auto => {
                // TODO
                0.0
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssomAtRule {}

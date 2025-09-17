use super::CascadeOrd;
use super::Num;
use super::Origin;
use super::Parser;
use super::Rule;
use super::StyleRule;
use super::StyleSheet;
use super::Token;
use super::Tokenizer;
use super::style::*;
use crate::arena::Arena;
use crate::arena::NodeId;
use crate::render::Color;
use crate::render::RenderArena;
use crate::render::RenderNodeType;
use std::ops::Deref;
use std::sync::LazyLock;

#[derive(Clone, Debug)]
pub struct CssomArena {
    roots: Vec<NodeId>,
    arena: Arena<CssStyleRule>,
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
                    for s in CssStyleRule::from_stylerule(stylerule, origin, false) {
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
    type Target = Arena<CssStyleRule>;

    fn deref(&self) -> &Self::Target {
        &self.arena
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssStyleRule {
    selector: Selector,
    style: CssStyle,
}

impl CssStyleRule {
    pub fn selector(&self) -> &Selector {
        &self.selector
    }

    pub fn style(&self) -> &CssStyle {
        &self.style
    }

    pub fn from_stylerule(stylerule: &StyleRule, origin: Origin, important: bool) -> Vec<Self> {
        if let Some(selectors) = Selector::from_tokens(stylerule.prelude(), origin, important)
            && let Some(style) = CssStyle::from_tokens(stylerule.block())
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
pub struct CssomAtRule {}

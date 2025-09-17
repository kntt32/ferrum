use super::Num;
use super::Token;
use crate::render::AlphaColor;
use crate::render::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct CssStyle {
    pub background_color: Option<BackgroundColor>,
    pub color: Option<ForegroundColor>,
    pub font_size: Option<FontSize>,
    pub margin_top: Option<Margin>,
    pub margin_right: Option<Margin>,
    pub margin_bottom: Option<Margin>,
    pub margin_left: Option<Margin>,
    pub width: Option<Width>,
    pub height: Option<Height>,
}

impl Default for CssStyle {
    fn default() -> Self {
        Self {
            background_color: Some(BackgroundColor::Color(AlphaColor::TRANSPARENT)),
            color: Some(ForegroundColor::Color(Color::BLACK.into())),
            font_size: Some(FontSize::Length(Length::Pixel(Num::Integer(16)))),
            margin_top: Some(Margin::Length(Length::Pixel(Num::Integer(16)))),
            margin_right: Some(Margin::Length(Length::Pixel(Num::Integer(16)))),
            margin_bottom: Some(Margin::Length(Length::Pixel(Num::Integer(16)))),
            margin_left: Some(Margin::Length(Length::Pixel(Num::Integer(16)))),
            width: Some(Width::Auto),
            height: Some(Height::Auto),
        }
    }
}

impl CssStyle {
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
                    this.background_color = BackgroundColor::from(iter).or(this.background_color);
                }
                "background" => {
                    if let Some(bg) = BackgroundColor::from(iter) {
                        this.background_color = Some(bg);
                    }
                }
                "color" => {
                    this.color = ForegroundColor::from(iter).or(this.color);
                }
                "font-size" => {
                    this.font_size = FontSize::from(iter).or(this.font_size);
                }
                "margin-top" => {
                    this.margin_top = Margin::from(iter).or(this.margin_top);
                }
                "margin-right" => {
                    this.margin_right = Margin::from(iter).or(this.margin_right);
                }
                "margin-bottom" => {
                    this.margin_bottom = Margin::from(iter).or(this.margin_bottom);
                }
                "margin-left" => {
                    this.margin_left = Margin::from(iter).or(this.margin_left);
                }
                "margin" => {
                    if let Some((top, right, bottom, left)) = Margin::from_margins(iter) {
                        this.margin_top = Some(top);
                        this.margin_right = Some(right);
                        this.margin_bottom = Some(bottom);
                        this.margin_left = Some(left);
                    }
                }
                "width" => {
                    this.width = Width::from(iter).or(this.width);
                }
                "height" => {
                    this.height = Height::from(iter).or(this.height);
                }
                _ => continue,
            }
        }

        Some(this)
    }

    pub fn inherit(&self) -> Self {
        let mut this = self.clone();

        this.background_color = None;
        this.margin_top = Some(Margin::Length(Length::Pixel(Num::Integer(0))));
        this.margin_right = Some(Margin::Length(Length::Pixel(Num::Integer(0))));
        this.margin_bottom = Some(Margin::Length(Length::Pixel(Num::Integer(0))));
        this.margin_left = Some(Margin::Length(Length::Pixel(Num::Integer(0))));
        this.width = Some(Width::Auto);
        this.height = Some(Height::Auto);

        this
    }

    pub fn apply_from(&mut self, from: &Self) {
        self.background_color = from.background_color.or(self.background_color);
        self.color = from.color.or(self.color);
        self.font_size = from.font_size.or(self.font_size);
        self.margin_top = from.margin_top.or(self.margin_top);
        self.margin_left = from.margin_left.or(self.margin_left);
        self.margin_bottom = from.margin_bottom.or(self.margin_bottom);
        self.margin_right = from.margin_right.or(self.margin_right);
        self.width = from.width.or(self.width);
        self.height = from.height.or(self.height);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ForegroundColor {
    Color(AlphaColor),
}

impl ForegroundColor {
    pub fn from<'a>(mut iter: impl Iterator<Item = &'a Token>) -> Option<Self> {
        match iter.next()? {
            Token::Ident(name) => Some(Self::Color(AlphaColor::from_name(name).ok()?)),
            Token::Hash { value, .. } => {
                Some(Self::Color(AlphaColor::from_str_noprefix(value).ok()?))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BackgroundColor {
    Color(AlphaColor),
}

impl BackgroundColor {
    pub fn from<'a>(mut iter: impl Iterator<Item = &'a Token>) -> Option<Self> {
        match iter.next()? {
            Token::Ident(name) => Some(Self::Color(AlphaColor::from_name(name).ok()?)),
            Token::Hash { value, .. } => {
                Some(Self::Color(AlphaColor::from_str_noprefix(value).ok()?))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Margin {
    Length(Length),
    Auto,
}

impl Margin {
    pub fn from<'a>(mut iter: impl Iterator<Item = &'a Token>) -> Option<Self> {
        match iter.next()? {
            Token::Dimension { value, unit } => Some(Self::Length(Length::from(*value, unit)?)),
            Token::Ident(ident) if ident.as_str() == "auto" => Some(Self::Auto),
            _ => None,
        }
    }

    pub fn from_margins<'a>(
        mut iter: impl Iterator<Item = &'a Token>,
    ) -> Option<(Self, Self, Self, Self)> {
        let mut margins = Vec::new();
        for _ in 0..4 {
            margins.push(Self::from(&mut iter)?);
        }

        match margins.len() {
            0 => None,
            1 => Some((
                margins[0].clone(),
                margins[0].clone(),
                margins[0].clone(),
                margins[0].clone(),
            )),
            2 => Some((
                margins[0].clone(),
                margins[1].clone(),
                margins[0].clone(),
                margins[1].clone(),
            )),
            3 => Some((
                margins[0].clone(),
                margins[1].clone(),
                margins[2].clone(),
                margins[1].clone(),
            )),
            4 => Some((
                margins[0].clone(),
                margins[1].clone(),
                margins[2].clone(),
                margins[3].clone(),
            )),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Width {
    Length(Length),
    Auto,
}

impl Width {
    pub fn from<'a>(mut iter: impl Iterator<Item = &'a Token>) -> Option<Self> {
        match iter.next()? {
            Token::Dimension { value, unit } => Some(Self::Length(Length::from(*value, unit)?)),
            Token::Ident(ident) if ident.as_str() == "auto" => Some(Self::Auto),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Height {
    Length(Length),
    Auto,
}

impl Height {
    pub fn from<'a>(mut iter: impl Iterator<Item = &'a Token>) -> Option<Self> {
        match iter.next()? {
            Token::Dimension { value, unit } => Some(Self::Length(Length::from(*value, unit)?)),
            Token::Ident(ident) if ident.as_str() == "auto" => Some(Self::Auto),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontSize {
    Length(Length),
    Auto,
}

impl FontSize {
    pub fn from<'a>(mut iter: impl Iterator<Item = &'a Token>) -> Option<Self> {
        match iter.next()? {
            Token::Dimension { value, unit } => Some(Self::Length(Length::from(*value, unit)?)),
            Token::Ident(ident) if ident.as_str() == "auto" => Some(Self::Auto),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Pixel(Num),
    Em(Num),
    Auto,
}

impl Length {
    pub fn from(num: Num, unit: &str) -> Option<Self> {
        match unit {
            "px" => Some(Self::Pixel(num)),
            "em" => Some(Self::Em(num)),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }
    /*
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
    }*/
}

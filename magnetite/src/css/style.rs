use super::Num;
use super::Token;
use crate::arena::NodeId;
use crate::render::AlphaColor;
use crate::render::Color;
use crate::render::RenderArena;

#[derive(Clone, Debug, PartialEq)]
pub struct CssStyle {
    pub display: Option<Display>,
    pub background_color: Option<BackgroundColor>,
    pub color: Option<ForegroundColor>,
    pub font_size: Option<FontSize>,
    pub margin_top: Option<Margin>,
    pub margin_right: Option<Margin>,
    pub margin_bottom: Option<Margin>,
    pub margin_left: Option<Margin>,
    pub padding_top: Option<Padding>,
    pub padding_right: Option<Padding>,
    pub padding_bottom: Option<Padding>,
    pub padding_left: Option<Padding>,
    pub width: Option<Width>,
    pub height: Option<Height>,
}

impl Default for CssStyle {
    fn default() -> Self {
        Self {
            display: Some(Display::Block),
            background_color: Some(BackgroundColor::Color(AlphaColor::TRANSPARENT)),
            color: Some(ForegroundColor::Color(Color::BLACK.into())),
            font_size: Some(FontSize::Length(Length::Pixel(Num::Integer(16)))),
            margin_top: Some(Margin::Length(Length::Pixel(Num::Integer(0)))),
            margin_right: Some(Margin::Length(Length::Pixel(Num::Integer(0)))),
            margin_bottom: Some(Margin::Length(Length::Pixel(Num::Integer(0)))),
            margin_left: Some(Margin::Length(Length::Pixel(Num::Integer(0)))),
            padding_top: Some(Padding::Length(Length::Pixel(Num::Integer(0)))),
            padding_right: Some(Padding::Length(Length::Pixel(Num::Integer(0)))),
            padding_bottom: Some(Padding::Length(Length::Pixel(Num::Integer(0)))),
            padding_left: Some(Padding::Length(Length::Pixel(Num::Integer(0)))),
            width: Some(Width::Auto),
            height: Some(Height::Auto),
        }
    }
}

impl CssStyle {
    pub fn new() -> Self {
        Self {
            display: None,
            background_color: None,
            color: None,
            font_size: None,
            margin_top: None,
            margin_right: None,
            margin_bottom: None,
            margin_left: None,
            padding_top: None,
            padding_right: None,
            padding_bottom: None,
            padding_left: None,
            width: None,
            height: None,
        }
    }

    pub fn from_tokens(tokens: &[Token]) -> Option<Self> {
        let mut this = Self::new();

        let len = tokens.len();
        let mut i = 0;
        while i + 2 < tokens.len() {
            let next_i =
                i + tokens[i..].partition_point(|token| !matches!(token, Token::Semicolon)) + 1;

            let Token::Ident(ref key) = tokens[i] else {
                i = next_i;
                continue;
            };

            if tokens[i + 1] != Token::Colon {
                i = next_i;
                continue;
            }

            let value = &tokens[i + 2..next_i - 1];

            match key.as_str() {
                "display" => {
                    if let Some(display) = Display::from(value) {
                        this.display = Some(display);
                    }
                }
                "background" => {
                    if let Some(background_color) = BackgroundColor::from(value) {
                        this.background_color = Some(background_color);
                    }
                }
                "background-color" => {
                    this.background_color = BackgroundColor::from(value).or(this.background_color);
                }
                "color" => {
                    this.color = ForegroundColor::from(value).or(this.color);
                }
                "font-size" => {
                    this.font_size = FontSize::from(value).or(this.font_size);
                }
                "margin-top" => {
                    this.margin_top = Margin::from(value).or(this.margin_top);
                }
                "margin-right" => {
                    this.margin_right = Margin::from(value).or(this.margin_right);
                }
                "margin-bottom" => {
                    this.margin_bottom = Margin::from(value).or(this.margin_bottom);
                }
                "margin-left" => {
                    this.margin_left = Margin::from(value).or(this.margin_left);
                }
                "margin" => {
                    if let Some((top, right, bottom, left)) = Margin::from_margins(value) {
                        this.margin_top = Some(top);
                        this.margin_right = Some(right);
                        this.margin_bottom = Some(bottom);
                        this.margin_left = Some(left);
                    }
                }
                "padding-top" => {
                    this.padding_top = Padding::from(value).or(this.padding_top);
                }
                "padding-right" => {
                    this.padding_right = Padding::from(value).or(this.padding_right);
                }
                "padding-bottom" => {
                    this.padding_bottom = Padding::from(value).or(this.padding_bottom);
                }
                "padding-left" => {
                    this.padding_left = Padding::from(value).or(this.padding_left);
                }
                "padding" => {
                    if let Some((top, right, bottom, left)) = Padding::from_paddings(value) {
                        this.padding_top = Some(top);
                        this.padding_right = Some(right);
                        this.padding_bottom = Some(bottom);
                        this.padding_left = Some(left);
                    }
                }
                "width" => {
                    this.width = Width::from(value).or(this.width);
                }
                "height" => {
                    this.height = Height::from(value).or(this.height);
                }
                _ => {}
            }

            i = next_i;
        }

        Some(this)
    }

    pub fn inherit(&self) -> Self {
        let mut this = Self::default();
        this.font_size = self.font_size;
        this
    }

    pub fn apply_from(&mut self, from: &Self) {
        self.display = from.display.or(self.display);
        self.background_color = from.background_color.or(self.background_color);
        self.color = from.color.or(self.color);
        self.font_size = from.font_size.or(self.font_size);
        self.margin_top = from.margin_top.or(self.margin_top);
        self.margin_left = from.margin_left.or(self.margin_left);
        self.margin_bottom = from.margin_bottom.or(self.margin_bottom);
        self.margin_right = from.margin_right.or(self.margin_right);
        self.padding_top = from.padding_top.or(self.padding_top);
        self.padding_right = from.padding_right.or(self.padding_right);
        self.padding_bottom = from.padding_bottom.or(self.padding_bottom);
        self.padding_left = from.padding_left.or(self.padding_left);
        self.width = from.width.or(self.width);
        self.height = from.height.or(self.height);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Display {
    Block,
    Inline,
    InlineBlock,
}

impl Display {
    pub fn from(tokens: &[Token]) -> Option<Self> {
        if let Token::Ident(ident) = tokens.first()? {
            match ident.as_str() {
                "block" => Some(Self::Block),
                "inline" => Some(Self::Inline),
                "inline-block" => Some(Self::InlineBlock),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ForegroundColor {
    Color(AlphaColor),
}

impl ForegroundColor {
    pub fn from(tokens: &[Token]) -> Option<Self> {
        match tokens.first()? {
            Token::Ident(name) => Some(Self::Color(AlphaColor::from_name(name).ok()?)),
            Token::Hash { value, .. } => {
                Some(Self::Color(AlphaColor::from_str_noprefix(value).ok()?))
            }
            _ => None,
        }
    }

    pub fn compute(&self) -> ComputedValue<AlphaColor> {
        match self {
            Self::Color(color) => ComputedValue::Value(*color),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BackgroundColor {
    Color(AlphaColor),
}

impl BackgroundColor {
    pub fn from(tokens: &[Token]) -> Option<Self> {
        match tokens.first()? {
            Token::Ident(name) => Some(Self::Color(AlphaColor::from_name(name).ok()?)),
            Token::Hash { value, .. } => {
                Some(Self::Color(AlphaColor::from_str_noprefix(value).ok()?))
            }
            _ => None,
        }
    }

    pub fn compute(&self) -> ComputedValue<AlphaColor> {
        match self {
            Self::Color(color) => ComputedValue::Value(*color),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Margin {
    Length(Length),
    Auto,
}

impl Margin {
    pub fn from(tokens: &[Token]) -> Option<Self> {
        match tokens.first()? {
            Token::Dimension { value, unit } => Some(Self::Length(Length::from(*value, unit)?)),
            Token::Ident(ident) if ident.as_str() == "auto" => Some(Self::Auto),
            _ => None,
        }
    }

    pub fn from_margins(tokens: &[Token]) -> Option<(Self, Self, Self, Self)> {
        let mut margins = Vec::new();
        for i in 0..tokens.len().min(4) {
            margins.push(Self::from(&tokens[i..])?);
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
pub enum Padding {
    Length(Length),
}

impl Padding {
    pub fn from(tokens: &[Token]) -> Option<Self> {
        match tokens.first()? {
            Token::Dimension { value, unit } => Some(Self::Length(Length::from(*value, unit)?)),
            _ => None,
        }
    }

    pub fn from_paddings(tokens: &[Token]) -> Option<(Self, Self, Self, Self)> {
        let mut paddings = Vec::new();
        for i in 0..tokens.len().min(4) {
            paddings.push(Self::from(&tokens[i..])?);
        }

        match paddings.len() {
            0 => None,
            1 => Some((
                paddings[0].clone(),
                paddings[0].clone(),
                paddings[0].clone(),
                paddings[0].clone(),
            )),
            2 => Some((
                paddings[0].clone(),
                paddings[1].clone(),
                paddings[0].clone(),
                paddings[1].clone(),
            )),
            3 => Some((
                paddings[0].clone(),
                paddings[1].clone(),
                paddings[2].clone(),
                paddings[1].clone(),
            )),
            4 => Some((
                paddings[0].clone(),
                paddings[1].clone(),
                paddings[2].clone(),
                paddings[3].clone(),
            )),
            _ => unreachable!(),
        }
    }

    pub fn compute(&self, render_arena: &RenderArena, id: NodeId) -> ComputedValue<f32> {
        match self {
            Self::Length(length) => length.compute(render_arena, id),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Width {
    Length(Length),
    Auto,
}

impl Width {
    pub fn from(tokens: &[Token]) -> Option<Self> {
        match tokens.first()? {
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
    pub fn from(tokens: &[Token]) -> Option<Self> {
        match tokens.first()? {
            Token::Dimension { value, unit } => Some(Self::Length(Length::from(*value, unit)?)),
            Token::Ident(ident) if ident.as_str() == "auto" => Some(Self::Auto),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontSize {
    Length(Length),
}

impl FontSize {
    pub fn from(tokens: &[Token]) -> Option<Self> {
        match tokens.first()? {
            Token::Dimension { value, unit } => Some(Self::Length(Length::from(*value, unit)?)),
            _ => None,
        }
    }

    pub fn compute(&self, render_arena: &RenderArena, id: NodeId) -> ComputedValue<f32> {
        match self {
            Self::Length(length) => length.compute(render_arena, id),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Pixel(Num),
    Em(Num),
}

impl Length {
    pub fn from(num: Num, unit: &str) -> Option<Self> {
        match unit {
            "px" => Some(Self::Pixel(num)),
            "em" => Some(Self::Em(num)),
            _ => None,
        }
    }

    pub fn compute(&self, render_arena: &RenderArena, id: NodeId) -> ComputedValue<f32> {
        match self {
            Self::Pixel(num) => ComputedValue::Value(num.as_floating() as f32),
            Self::Em(num) => {
                let parent_size = if let Some(parent_id) = render_arena[id].parent() {
                    render_arena[parent_id].style().font_size
                } else {
                    16.0
                };
                ComputedValue::Value(parent_size * num.as_floating() as f32)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComputedValue<T> {
    Value(T),
    Auto,
}

impl<T> ComputedValue<T> {
    pub fn is_value(&self) -> bool {
        matches!(self, Self::Value(..))
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    pub fn unwrap(self) -> T {
        if let Self::Value(value) = self {
            value
        } else {
            panic!("called ComputedValue::unwrap on a Auto value")
        }
    }
}

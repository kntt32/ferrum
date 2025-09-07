use super::color::Color;
use super::renderer::Buff;
use ab_glyph::Font as AbFont;
use ab_glyph::FontRef;
use ab_glyph::Glyph;
use ab_glyph::Rect;
use ab_glyph::ScaleFont as AbScaleFont;
use std::sync::LazyLock;

static DEFAULT_FONT: LazyLock<Font<FontRef<'static>>> = LazyLock::new(|| {
    Font::new(
        FontRef::try_from_slice(include_bytes!(
            "../../../assets/fonts/NotoSansJP-VariableFont_wght.ttf"
        ))
        .unwrap(),
    )
});

#[derive(Clone, Debug)]
pub struct Font<F: AbFont> {
    font: F,
}

impl Font<FontRef<'static>> {
    pub fn default() -> &'static Self {
        LazyLock::force(&DEFAULT_FONT)
    }
}

impl<F: AbFont> Font<F> {
    pub fn new(font: F) -> Self {
        Self { font }
    }

    pub fn glyph(&self, c: char, size: f32) -> Glyph {
        self.font.glyph_id(c).with_scale(size)
    }

    pub fn layout(&self, glyph: &Glyph) -> Layout {
        Layout::from_rect(self.font.glyph_bounds(glyph))
    }

    pub fn advance(&self, glyph: &Glyph) -> Advance {
        let scaled_font = self.font.as_scaled(glyph.scale);
        let horz = scaled_font.h_advance(glyph.id);
        let vert = scaled_font.v_advance(glyph.id);
        Advance { horz, vert }
    }

    pub fn draw(&self, glyph: Glyph, buffer: &mut impl Buff, x: usize, y: usize, color: Color) {
        if let Some(outline_glyph) = self.font.outline_glyph(glyph) {
            outline_glyph.draw(|x, y, alpha| {
                if let Some(src) = buffer.get_mut(x as usize, y as usize) {
                    *src = color.alpha(alpha, Color::from_u32(*src)).as_u32();
                }
            });
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Advance {
    pub horz: f32,
    pub vert: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Layout {
    pub fn from_rect(rect: Rect) -> Self {
        let Rect { min, max } = rect;
        let x = min.x;
        let y = min.y;
        let width = max.x - x;
        let height = max.y - y;
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

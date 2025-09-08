use super::buff::Buff;
use super::color::Color;
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

    pub fn glyph_str(&self, s: &str, size: f32) -> Vec<Glyph> {
        let mut glyphs = Vec::new();
        glyphs.reserve(s.chars().count());

        for c in s.chars() {
            glyphs.push(self.glyph(c, size));
        }

        glyphs
    }

    pub fn layout_str(&self, glyphs: &[Glyph]) -> Layout {
        let mut x = 0.0f32;
        let mut y = 0.0f32;
        let mut width = 0.0;
        let mut height = 0.0;
        let mut draw_x = 0.0;
        let mut draw_y = 0.0;

        for glyph in glyphs {
            let layout = self.layout(glyph);
            x = x.min(layout.x + draw_x);
            y = y.min(layout.y + draw_y);
            width = draw_x + layout.width;
            height = draw_y + layout.height;

            let advance = self.advance(glyph);
            draw_x += advance.horz;
            draw_y += advance.vert;
        }

        Layout {
            x,
            y,
            width,
            height,
        }
    }

    pub fn advance_str(&self, glyphs: &[Glyph]) -> Advance {
        let mut horz = 0.0;
        let mut vert = 0.0;

        for glyph in glyphs {
            let advance = self.advance(glyph);
            horz += advance.horz;
            vert += advance.vert;
        }

        Advance { horz, vert }
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
            outline_glyph.draw(|draw_rel_x, draw_rel_y, alpha| {
                if let Some(src) = buffer.get_mut(
                    x as isize + draw_rel_x as isize,
                    y as isize + draw_rel_y as isize,
                ) {
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

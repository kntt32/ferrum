use super::Buff;
use super::Color;
use super::Font;
use super::RenderArena;
use super::RenderNodeType;
use super::RenderStyle;
use crate::arena::NodeId;
use crate::color;

#[derive(Clone, Debug)]
pub struct Renderer {
    arena: RenderArena,
}

impl Renderer {
    pub const BACKGROUND: Color = Color::WHITE;

    pub fn new(arena: RenderArena) -> Self {
        Self { arena }
    }

    pub fn render(&self, buff: &mut impl Buff) {
        buff.fill(Self::BACKGROUND);

        self.render_node(buff, RenderArena::ROOT);
    }

    fn render_node(&self, buff: &mut impl Buff, id: NodeId) {
        let style = self.arena[id].style();

        // TODO: remove this by background color support of css
        buff.draw_rect_border(
            style.x(),
            style.y(),
            style.width(),
            style.height(),
            color!(#eeeeee),
        );

        match self.arena[id].node_type() {
            RenderNodeType::Element { .. } => {
                for child in self.arena.children(id) {
                    self.render_node(buff, child);
                }
            }
            RenderNodeType::Text(text) => {
                let font = Font::default();
                let glyphs = font.glyph_str(&text, style.font_size());
                let layout = font.layout_str(&glyphs);
                font.draw_str(
                    glyphs,
                    buff,
                    style.x() - layout.x as isize,
                    style.y() - layout.y as isize,
                    style.color(),
                );
            }
        }
    }
}

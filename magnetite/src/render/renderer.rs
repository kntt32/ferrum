use super::AlphaColor;
use super::Buff;
use super::Color;
use super::Font;
use super::LayoutArena;
use super::LayoutBox;
use super::LayoutType;
use super::LineFlagment;
use super::RenderArena;
use super::RenderNodeType;
use super::Window;
use crate::arena::NodeId;

#[derive(Clone, Debug)]
pub struct Renderer {
    render_arena: RenderArena,
    layout_arena: LayoutArena,
}

impl Renderer {
    pub const BACKGROUND: Color = Color::WHITE;

    pub fn new(render_arena: RenderArena, layout_arena: LayoutArena) -> Self {
        Self {
            render_arena,
            layout_arena,
        }
    }

    pub fn render(&self, buff: &mut impl Buff) {
        buff.fill(Self::BACKGROUND);

        self.render_layout_box(0, buff, 0, 0);
    }

    fn render_layout_box<'a>(
        &self,
        layout_box_id: NodeId,
        origin_buff: &mut impl Buff,
        x: isize,
        y: isize,
    ) {
        let layout_box = &self.layout_arena[layout_box_id];
        let layout = &layout_box.layout;
        {
            let mut buff = origin_buff.window(
                x + layout.margin_left.unwrap() as isize,
                y + layout.margin_top.unwrap() as isize,
                layout.padding_left + layout.width.unwrap() + layout.padding_right,
                layout.padding_top + layout.height.unwrap() + layout.padding_bottom,
            );

            match &layout_box.node_type {
                LayoutType::Block => {
                    if let Some(render_node_id) = layout_box.render_arena_id {
                        let style = &self.render_arena[render_node_id].style.as_ref().unwrap();
                        buff.fill(style.background_color);
                    }
                }
                LayoutType::Line => {}
                LayoutType::Fragment(fragment) => {
                    let render_node_id = layout_box.render_arena_id.unwrap();
                    let style = &self.render_arena[render_node_id].style.as_ref().unwrap();

                    buff.fill(style.background_color);

                    match fragment {
                        LineFlagment::Text(text) => {
                            let font = Font::default();
                            let glyphs = font.glyph_str(text.as_str(), style.font_size);
                            let layout = font.layout_str(&glyphs);
                            font.draw_str(
                                glyphs,
                                &mut buff,
                                -layout.x as isize,
                                -layout.y as isize,
                                style.color,
                            );
                        }
                    }
                }
            }
        }
        for child in self.layout_arena.children(layout_box_id) {
            let child_layout = self.layout_arena[child].layout;
            self.render_layout_box(
                child,
                origin_buff,
                x + layout.padding_left as isize
                    + layout.margin_left.unwrap() as isize
                    + child_layout.x.unwrap(),
                y + layout.padding_top as isize
                    + layout.margin_top.unwrap() as isize
                    + child_layout.y.unwrap(),
            );
        }
    }
}

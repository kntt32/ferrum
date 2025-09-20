use super::Font;
use super::GlyphLayout;
use super::RenderArena;
use super::RenderNodeType;
use crate::arena::Arena;
use crate::arena::NodeId;
use crate::css::ComputedValue;
use crate::css::Display;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct LayoutArena {
    arena: Arena<LayoutBox>, // Block->Block->...
                             //      ->Flagment
}

impl LayoutArena {
    pub fn new(render_arena: &RenderArena, containing_width: f32) -> Self {
        let mut this = Self {
            arena: Arena::with_root(LayoutBox::body(render_arena, containing_width as usize)),
        };

        this.build(0, render_arena, 0, containing_width);

        this
    }

    fn build(
        &mut self,
        parent_id: NodeId,
        render_arena: &RenderArena,
        render_arena_id: NodeId,
        containing_width: f32,
    ) {
        let render_node = &render_arena[render_arena_id];

        match (
            render_node.css_style.display.unwrap(),
            render_node.node_type.is_replace_element(),
        ) {
            (Display::Block, false) => {
                self.build_block_unreplace_element(parent_id, render_arena, render_arena_id);
            }
            (Display::Inline, false) => {
                self.build_inline_unreplace_element(parent_id, render_arena, render_arena_id);
            }
            _ => todo!(),
        };
    }

    fn calc_block_nonreplaced_width_and_horizontal_margin(
        &self,
        render_arena: &RenderArena,
        render_arena_id: NodeId,
        containing_width: f32,
    ) -> (f32, f32, f32) {
        let width: f32;
        let margin_left: f32;
        let margin_right: f32;

        let style = &render_arena[render_arena_id].style.as_ref().unwrap();
        let properties = [style.margin_left, style.width, style.margin_right];
        let sum = style.padding_left
            + style.margin_left.unwrap_or(0.0)
            + style.width.unwrap_or(0.0)
            + style.margin_right.unwrap_or(0.0)
            + style.padding_right;

        if !style.width.is_auto() && containing_width < sum {
            width = style.width.unwrap();
            margin_left = style.margin_left.unwrap_or(0.0);
            margin_right = style.margin_right.unwrap_or(0.0);
        } else if !properties.contains(&ComputedValue::Auto) && containing_width < sum {
            width = style.width.unwrap();
            margin_left = style.margin_left.unwrap();
            margin_right = containing_width
                - (style.padding_left
                    + style.margin_left.unwrap()
                    + style.width.unwrap()
                    + style.padding_right);
        } else if properties.iter().filter(|value| value.is_auto()).count() == 1 {
            let remaining = containing_width - sum;
            width = style.width.unwrap_or(remaining);
            margin_left = style.margin_left.unwrap_or(remaining);
            margin_right = style.margin_right.unwrap_or(remaining);
        } else if style.width.is_auto() {
            let remaining = containing_width - sum;
            width = remaining;
            margin_left = style.margin_left.unwrap_or(0.0);
            margin_right = style.margin_right.unwrap_or(0.0);
        } else if style.margin_left.is_auto() && style.margin_right.is_auto() {
            let remaining = containing_width - sum;
            width = style.width.unwrap();
            margin_left = remaining / 2.0;
            margin_right = remaining / 2.0;
        } else {
            width = style.width.unwrap();
            margin_left = style.margin_left.unwrap();
            margin_right = style.margin_right.unwrap();
        }

        (width, margin_left, margin_right)
    }

    fn calc_block_nonreplaced_height_and_vertical_margin(
        &self,
        node_id: NodeId,
        render_arena: &RenderArena,
        render_arena_id: NodeId,
    ) -> (f32, f32, f32) {
        let style = &render_arena[render_arena_id].style.as_ref().unwrap();

        let margin_top = style.margin_top.unwrap_or(0.0);
        let margin_bottom = style.margin_bottom.unwrap_or(0.0);

        let height = if style.height.is_auto() {
            self.children(node_id)
                .map(|id: NodeId| self[id].layout.height.unwrap())
                .sum::<usize>() as f32
        } else {
            style.height.unwrap()
        };

        (height, margin_top, margin_bottom)
    }

    fn build_block_unreplace_element(
        &mut self,
        parent_id: NodeId,
        render_arena: &RenderArena,
        render_arena_id: NodeId,
    ) {
        let containing_width = self[parent_id].layout.width.unwrap();

        match render_arena[render_arena_id].node_type {
            RenderNodeType::Element {
                ref name,
                ref attributes,
            } => {
                let (width, margin_left, margin_right) = self
                    .calc_block_nonreplaced_width_and_horizontal_margin(
                        render_arena,
                        render_arena_id,
                        containing_width as f32,
                    );
                let layout_box = LayoutBox {
                    node_type: LayoutType::Block,
                    layout: Layout {
                        x: None,
                        y: None,
                        width: Some(width as usize),
                        height: None,
                        margin_top: None,
                        margin_right: Some(margin_right as usize),
                        margin_bottom: None,
                        margin_left: Some(margin_left as usize),
                    },
                    render_arena_id,
                };
                let node_id = self.arena.insert_child(parent_id, layout_box);

                for child in render_arena.children(render_arena_id).collect::<Vec<_>>() {
                    self.build(node_id, render_arena, child, width);
                }

                let x = 0;
                let mut y = 0;
                for child in self.children(node_id).collect::<Vec<_>>() {
                    self.arena[child].layout.x = Some(x);
                    self.arena[child].layout.y = Some(y);
                    y += self.arena[child].layout.height.unwrap() as isize;
                }

                let (height, margin_top, margin_bottom) = self
                    .calc_block_nonreplaced_height_and_vertical_margin(
                        node_id,
                        render_arena,
                        render_arena_id,
                    );

                self.arena[node_id].layout.height = Some(height as usize);
                self.arena[node_id].layout.margin_top = Some(margin_top as usize);
                self.arena[node_id].layout.margin_bottom = Some(margin_bottom as usize);
            }
            RenderNodeType::Text(ref text) => unreachable!(),
        }
    }

    fn build_inline_unreplace_element(
        &mut self,
        parent_id: NodeId,
        render_arena: &RenderArena,
        render_arena_id: NodeId,
    ) {
        match render_arena[render_arena_id].node_type {
            RenderNodeType::Element { .. } => {
                todo!("{:?}", render_arena[render_arena_id].css_style.display);
            }
            RenderNodeType::Text(ref text) => {
                let font_size = render_arena[render_arena_id].style.unwrap().font_size;
                let font = Font::default();
                let glyphs = font.glyph_str(text.as_str(), font_size);
                let GlyphLayout {
                    x,
                    y,
                    width,
                    height,
                } = font.layout_str(&glyphs);
                let node_type = LayoutType::Fragment(LineFlagment::Text {
                    text: text.clone(),
                    render_arena_id,
                });
                let layout_box = LayoutBox {
                    node_type,
                    layout: Layout {
                        x: None,
                        y: None,
                        width: Some((width - x) as usize),
                        height: Some((height - y) as usize), // TODO
                        margin_top: None,
                        margin_right: None,
                        margin_bottom: None,
                        margin_left: None,
                    },
                    render_arena_id,
                };
                self.arena.insert_child(parent_id, layout_box);
            }
        }
    }
}

impl Deref for LayoutArena {
    type Target = Arena<LayoutBox>;

    fn deref(&self) -> &Self::Target {
        &self.arena
    }
}

#[derive(Clone, Debug)]
pub struct LayoutBox {
    node_type: LayoutType,
    layout: Layout,
    render_arena_id: NodeId,
}

impl LayoutBox {
    pub fn body(render_arena: &RenderArena, width: usize) -> Self {
        let render_node = &render_arena[0];
        let RenderNodeType::Element { .. } = render_node.node_type else {
            panic!("invalid render tree");
        };

        LayoutBox {
            node_type: LayoutType::Block,
            layout: Layout {
                x: None,
                y: None,
                width: Some(width),
                height: None,
                margin_top: Some(0),
                margin_right: Some(0),
                margin_bottom: Some(0),
                margin_left: Some(0),
            },
            render_arena_id: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum LayoutType {
    Block,
    Fragment(LineFlagment),
}

#[derive(Clone, Debug)]
pub enum LineFlagment {
    Text {
        text: String,
        render_arena_id: NodeId,
    },
    // Replacement{..},
}

impl LineFlagment {
    pub fn split(
        self,
        render_arena: &RenderArena,
        containing_width: usize,
        remaining_width: usize,
    ) -> (Self, Option<Self>) {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    x: Option<isize>,
    y: Option<isize>,
    width: Option<usize>,
    height: Option<usize>,
    margin_top: Option<usize>,
    margin_right: Option<usize>,
    margin_bottom: Option<usize>,
    margin_left: Option<usize>,
}

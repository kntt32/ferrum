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
                             //      ->Line->Flagment
}

impl LayoutArena {
    pub fn new(render_arena: &RenderArena, containing_width: f32) -> Self {
        let mut this = Self {
            arena: Arena::with_root(LayoutBox::root(containing_width as usize)),
        };

        this.build(0, render_arena, 0, containing_width);
        let body_id = this[0].child().unwrap();
        let body_layout = &mut this.arena[body_id].layout;
        body_layout.x = Some(0);
        body_layout.y = Some(0);
        this.arena[0].layout.height = Some(body_layout.height.unwrap());

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
                .map(|id: NodeId| {
                    let layout = &self[id].layout;
                    layout.margin_top.unwrap()
                        + layout.padding_top
                        + layout.height.unwrap()
                        + layout.padding_bottom
                        + layout.margin_bottom.unwrap()
                })
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
        let style = &render_arena[render_arena_id].style.as_ref().unwrap();

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
                        padding_top: style.padding_top as usize,
                        padding_right: style.padding_right as usize,
                        padding_bottom: style.padding_bottom as usize,
                        padding_left: style.padding_left as usize,
                    },
                    render_arena_id: Some(render_arena_id),
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
                    let layout = &self.arena[child].layout;
                    y += (layout.margin_top.unwrap()
                        + layout.padding_top
                        + layout.height.unwrap()
                        + layout.padding_bottom
                        + layout.margin_bottom.unwrap()) as isize;
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
                // todo!("{:?}", render_arena[render_arena_id].css_style.display);
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
                let node_type = LayoutType::Fragment(LineFlagment::Text(text.clone()));
                let mut layout_box = LayoutBox {
                    node_type,
                    layout: Layout {
                        x: None,
                        y: None,
                        width: Some(width as usize),
                        height: Some(height as usize), // TODO
                        margin_top: Some(0),
                        margin_right: Some(0),
                        margin_bottom: Some(0),
                        margin_left: Some(0),
                        padding_top: 0,
                        padding_right: 0,
                        padding_bottom: 0,
                        padding_left: 0,
                    },
                    render_arena_id: Some(render_arena_id),
                };

                if let Some(last_sibling) = self.children(parent_id).last()
                    && self[last_sibling].node_type == LayoutType::Line
                {
                    let layout = &mut layout_box.layout;
                    let line_layout = &mut self.arena[last_sibling].layout;
                    layout.x = Some(line_layout.width.unwrap() as isize);
                    layout.y = Some(0);
                    *line_layout.width.as_mut().unwrap() += layout.width.unwrap();
                    *line_layout.height.as_mut().unwrap() =
                        line_layout.height.unwrap().max(layout.height.unwrap());
                    self.arena.insert_child(last_sibling, layout_box);
                } else {
                    let layout = &mut layout_box.layout;
                    layout.x = Some(0);
                    layout.y = Some(0);
                    let line = LayoutBox::line(layout.width.unwrap(), layout.height.unwrap());
                    let line_id = self.arena.insert_child(parent_id, line);
                    self.arena.insert_child(line_id, layout_box);
                }
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
    pub node_type: LayoutType,
    pub layout: Layout,
    pub render_arena_id: Option<NodeId>,
}

impl LayoutBox {
    pub fn line(width: usize, height: usize) -> Self {
        Self {
            node_type: LayoutType::Line,
            layout: Layout {
                x: None,
                y: None,
                width: Some(width),
                height: Some(height),
                margin_top: Some(0),
                margin_right: Some(0),
                margin_bottom: Some(0),
                margin_left: Some(0),
                padding_top: 0,
                padding_right: 0,
                padding_bottom: 0,
                padding_left: 0,
            },
            render_arena_id: None,
        }
    }

    pub fn root(width: usize) -> Self {
        LayoutBox {
            node_type: LayoutType::Block,
            layout: Layout {
                x: Some(0),
                y: Some(0),
                width: Some(width),
                height: None,
                margin_top: Some(0),
                margin_right: Some(0),
                margin_bottom: Some(0),
                margin_left: Some(0),
                padding_top: 0,
                padding_right: 0,
                padding_bottom: 0,
                padding_left: 0,
            },
            render_arena_id: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LayoutType {
    Block,
    Line,
    Fragment(LineFlagment),
}

#[derive(Clone, Debug, PartialEq)]
pub enum LineFlagment {
    Text(String),
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
    pub x: Option<isize>,
    pub y: Option<isize>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub margin_top: Option<usize>,
    pub margin_right: Option<usize>,
    pub margin_bottom: Option<usize>,
    pub margin_left: Option<usize>,
    pub padding_top: usize,
    pub padding_right: usize,
    pub padding_bottom: usize,
    pub padding_left: usize,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: None,
            height: None,
            margin_top: None,
            margin_right: None,
            margin_bottom: None,
            margin_left: None,
            padding_top: 0,
            padding_right: 0,
            padding_bottom: 0,
            padding_left: 0,
        }
    }
}

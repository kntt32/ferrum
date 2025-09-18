use super::RenderArena;
use super::RenderNodeType;
use crate::arena::Arena;
use crate::arena::NodeId;
use crate::css::Display;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct LayoutArena {
    arena: Arena<LayoutBox>,
}

impl LayoutArena {
    pub fn new(render_arena: &RenderArena) -> Self {
        let mut this = Self {
            arena: Arena::new(),
        };

        this.build(render_arena, 0);

        this
    }

    fn build(&mut self, render_arena: &RenderArena, render_arena_id: NodeId) -> NodeId {
        let render_node = &render_arena[render_arena_id];

        let layout_box = match (
            render_node.css_style.display.unwrap(),
            render_node.node_type.is_replace_element(),
        ) {
            (Display::Block, false) => {
                todo!()
            }
            (Display::Inline, false) => {
                self.build_inline_unreplace_element(render_arena, render_arena_id)
            }
            _ => todo!(),
        };

        self.arena.push(layout_box)
    }

    fn build_inline_unreplace_element(
        &mut self,
        render_arena: &RenderArena,
        render_arena_id: NodeId,
    ) -> LayoutBox {
        match render_arena[render_arena_id].node_type {
            RenderNodeType::Element { .. } => {}
            RenderNodeType::Text(_) => {
                /*
                let mut fragments = Vec::new();

                LayoutBox {
                    node_type: LayoutType::Line(fragments),
                    layout,
                    render_tree_node_id: render_arena_id,
                }*/
            }
        }
        todo!()
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
    render_tree_node_id: NodeId,
}

#[derive(Clone, Debug)]
pub enum LayoutType {
    Block,
    Line(Vec<LineFlagment>),
}

#[derive(Clone, Debug)]
pub enum LineFlagment {
    Text(String),
    InlineBox(LayoutBox),
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    x: isize,
    y: isize,
    width: usize,
    height: usize,
}

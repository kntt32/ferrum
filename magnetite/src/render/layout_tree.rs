use super::RenderArena;
use crate::arena::Arena;
use crate::arena::NodeId;

#[derive(Clone, Debug)]
pub struct LayoutArena {
    arena: Arena<LayoutBox>,
}

impl LayoutArena {
    pub fn new(render_arena: &RenderArena) -> Self {
        let mut this = Self { arena: Arena::new() };
        this
    }
}

#[derive(Clone, Debug)]
pub struct LayoutBox {
    node_type: LayoutType,
    layout: Layout,
    render_tree_node_id: NodeId,
}

impl LayoutBox {
    pub fn build(render_arena: &RenderArena, id: NodeId, layout_arena: &LayoutArena, parent_box: Option<&Self>) -> Self {
        todo!()
    }
}

impl Default for LayoutBox {
    fn default() -> Self {
        Self {
            node_type: LayoutType::Element {},
            layout: Layout {x: 0, y: 0, width: 0, height: 0},
            render_tree_node_id: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum LayoutType {
    Element {},
    Line {},
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    x: isize,
    y: isize,
    width: usize,
    height: usize,
}

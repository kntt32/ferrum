use super::Color;
use crate::arena::Arena;
use crate::arena::NodeId;

pub struct RenderArena {
    arena: Arena<Node>,
}

pub struct Node {
    // node_type: NodeType,
    x: Option<usize>,
    y: Option<usize>,
    width: Option<usize>,
    height: Option<usize>,
    color: Option<Color>,
}

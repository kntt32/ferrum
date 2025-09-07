// mod render_builder;

use super::Color;
use crate::arena::Arena;
use crate::arena::NodeId;
use crate::html::DomArena;
use std::collections::HashMap;

type Node = RenderNode;
type NodeType = RenderNodeType;

#[derive(Clone, Debug)]
pub struct RenderArena {
    arena: Arena<Node>,
}

impl RenderArena {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderNode {
    node_type: NodeType,
    size: Option<usize>,
    x: Option<usize>,
    y: Option<usize>,
    width: Option<usize>,
    height: Option<usize>,
    color: Option<Color>,
}

#[derive(Clone, Debug)]
pub enum RenderNodeType {
    Element {
        name: String,
        attributes: HashMap<String, String>,
    },
    Text(String),
}

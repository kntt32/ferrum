use super::Font;
use super::Layout;
use crate::arena::Arena;
use crate::arena::NodeId;
use crate::html::DomArena;
use crate::html::DomNodeType;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

type Node = RenderNode;
type NodeType = RenderNodeType;

#[derive(Clone, Debug)]
pub struct RenderArena {
    arena: Arena<Node>,
}

impl RenderArena {
    pub const ROOT: NodeId = 0;

    pub fn new(dom: &DomArena) -> Self {
        let mut this = Self {
            arena: Arena::new(),
        };
        this.build_tree(dom);
        this.attach_style();

        this
    }

    fn build_tree(&mut self, dom: &DomArena) {
        for dom_child_id in dom.children(DomArena::DOCUMENT_IDX) {
            if let DomNodeType::Element { ref name, .. } = dom[dom_child_id].node_type
                && name == "html"
            {
                self.build_html(dom, dom_child_id);
                break;
            }
        }
    }

    fn build_html(&mut self, dom: &DomArena, dom_parent_id: NodeId) {
        for dom_child_id in dom.children(dom_parent_id) {
            if let DomNodeType::Element {
                ref name,
                ref attributes,
            } = dom[dom_child_id].node_type
                && name == "body"
            {
                self.arena.push(Node::new(NodeType::Element {
                    name: name.clone(),
                    attributes: attributes.clone(),
                }));
                self.build_body(dom, dom_child_id, 0);
                break;
            }
        }
    }

    fn build_body(&mut self, dom: &DomArena, dom_parent_id: NodeId, arena_parent_id: NodeId) {
        for dom_child_id in dom.children(dom_parent_id) {
            match dom[dom_child_id].node_type {
                DomNodeType::Element {
                    ref name,
                    ref attributes,
                } => {
                    let arena_child_id = self.arena.insert_child(
                        arena_parent_id,
                        RenderNode::new(RenderNodeType::Element {
                            name: name.clone(),
                            attributes: attributes.clone(),
                        }),
                    );
                    self.build_body(dom, dom_child_id, arena_child_id);
                }
                DomNodeType::String(ref s) => {
                    let text = s
                        .replace(|c: char| c.is_whitespace(), " ")
                        .replace("  ", " ");
                    if !text.is_empty() {
                        self.arena.insert_child(
                            arena_parent_id,
                            RenderNode::new(RenderNodeType::Text(text)),
                        );
                    }
                }
                ref nt => {
                    println!("IGNORED: {:?}", nt);
                }
            }
        }
    }

    fn attach_style(&mut self) {
        self.attach_style_for(0, 0, 0, 10.0);
    }

    fn attach_style_for(&mut self, id: NodeId, x: isize, mut y: isize, size: f32) {
        let style = &mut self.arena[id].style;
        style.x = Some(x);
        style.y = Some(y);
        style.size = Some(size);

        match &self.arena[id].node_type {
            NodeType::Element { .. } => {
                let mut width = 0;

                for child_id in self.arena.children(id).collect::<Vec<NodeId>>() {
                    self.attach_style_for(child_id, x, y, size);
                    y += self.arena[child_id].style.height.unwrap() as isize;
                    width = width.max(self.arena[child_id].style.width.unwrap());
                }

                let style = &mut self.arena[id].style;
                style.width = Some(width);
                style.height = Some((y - style.y()) as usize);
            }
            NodeType::Text(text) => {
                let style = &self.arena[id].style;
                let font = Font::default();
                let glyphs = font.glyph_str(&text, style.size());
                let Layout { width, height, .. } = font.layout_str(&glyphs);
                let style = &mut self.arena[id].style;
                style.width = Some(width as usize);
                style.height = Some(height as usize);
            }
        }
    }
}

impl Deref for RenderArena {
    type Target = Arena<Node>;

    fn deref(&self) -> &Self::Target {
        &self.arena
    }
}

impl DerefMut for RenderArena {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.arena
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RenderStyle {
    size: Option<f32>,
    x: Option<isize>,
    y: Option<isize>,
    width: Option<usize>,
    height: Option<usize>,
}

impl RenderStyle {
    pub fn new() -> Self {
        Self {
            size: None,
            x: None,
            y: None,
            width: None,
            height: None,
        }
    }

    pub fn size(&self) -> f32 {
        self.size.expect("RenderStyle.size should be initialized")
    }

    pub fn x(&self) -> isize {
        self.x.expect("RenderStyle.x should be initialized")
    }

    pub fn y(&self) -> isize {
        self.y.expect("RenderStyle.y should be initialized")
    }

    pub fn width(&self) -> usize {
        self.width.expect("RenderStyle.width should be initialized")
    }

    pub fn height(&self) -> usize {
        self.height
            .expect("RenderStyle.height should be initialized")
    }
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct RenderNode {
    node_type: NodeType,
    style: RenderStyle,
}

impl RenderNode {
    pub fn new(node_type: NodeType) -> Self {
        Self {
            node_type,
            style: RenderStyle::new(),
        }
    }

    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    pub fn style(&self) -> RenderStyle {
        self.style
    }

    pub fn body() -> Self {
        Self {
            node_type: RenderNodeType::Element {
                name: "body".to_string(),
                attributes: HashMap::new(),
            },
            style: RenderStyle::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum RenderNodeType {
    Element {
        name: String,
        attributes: HashMap<String, String>,
    },
    Text(String),
}

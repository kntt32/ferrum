use super::Font;
use super::Layout;
use crate::arena::Arena;
use crate::arena::NodeId;
use crate::css::CssomArena;
use crate::css::CssomStyle;
use crate::html::DomArena;
use crate::html::DomNodeType;
use crate::render::Color;
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
        let cssom = dom.cssom();
        this.attach_style(&cssom);

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
                    let mut text_chars = s.trim().chars().collect::<Vec<char>>();
                    text_chars.dedup_by(|c1, c2| c1.is_whitespace() && c2.is_whitespace());
                    let text = text_chars
                        .iter()
                        .collect::<String>()
                        .replace(|c: char| c.is_whitespace(), " ");
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

    fn attach_style(&mut self, cssom: &CssomArena) {
        self.attach_style_for(0, 0, 0, cssom);
    }

    fn attach_style_for(&mut self, id: NodeId, x: isize, y: isize, cssom: &CssomArena) {
        self.arena[id].style = if let Some(parent_id) = self.arena[id].parent() {
            self.arena[parent_id].style.inherit()
        } else {
            RenderStyle::body()
        };
        self.arena[id].style.x = Some(x);
        self.arena[id].style.y = Some(y);
        if let Some(cssom_style) = cssom.search_style_rule(self, id) {
            let style = &mut self.arena[id].style;
            style.attach_cssom_style(&cssom_style);
        }

        let x = self.arena[id].style.x.unwrap();
        let mut y = self.arena[id].style.y.unwrap();
        match &self.arena[id].node_type {
            NodeType::Element { .. } => {
                let mut width = 0;

                for child_id in self.arena.children(id).collect::<Vec<NodeId>>() {
                    self.attach_style_for(child_id, x, y, cssom);
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
                let glyphs = font.glyph_str(&text, style.font_size());
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
    pub font_size: Option<f32>,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    x: Option<isize>,
    y: Option<isize>,
    width: Option<usize>,
    height: Option<usize>,
}

impl RenderStyle {
    pub fn new() -> Self {
        Self {
            font_size: None,
            color: None,
            background_color: None,
            x: None,
            y: None,
            width: None,
            height: None,
        }
    }

    pub fn body() -> Self {
        Self {
            font_size: Some(10.0),
            color: Some(Color::BLACK),
            background_color: None,
            x: Some(0),
            y: Some(0),
            width: None,
            height: None,
        }
    }

    pub fn inherit(&self) -> Self {
        Self {
            font_size: self.font_size,
            color: self.color,
            background_color: None,
            x: None,
            y: None,
            width: None,
            height: None,
        }
    }

    pub fn attach_cssom_style(&mut self, cssom_style: &CssomStyle) {
        self.font_size = cssom_style
            .font_size
            .as_ref()
            .map(|v| v.as_pixel())
            .or(self.font_size);
        self.color = cssom_style.color.or(self.color);
        self.background_color = cssom_style.background_color.or(self.background_color);
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
            .expect("RenderStyle.font_size must be initialized")
    }

    pub fn color(&self) -> Color {
        self.color.expect("RenderStyle.color must be initialized")
    }

    pub fn x(&self) -> isize {
        self.x.expect("RenderStyle.x must be initialized")
    }

    pub fn y(&self) -> isize {
        self.y.expect("RenderStyle.y must be initialized")
    }

    pub fn width(&self) -> usize {
        self.width.expect("RenderStyle.width must be initialized")
    }

    pub fn height(&self) -> usize {
        self.height.expect("RenderStyle.height must be initialized")
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

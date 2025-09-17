use super::Font;
use super::Layout;
use crate::arena::Arena;
use crate::arena::NodeId;
use crate::css::CssStyle;
use crate::css::CssomArena;
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

    pub fn new(dom: &DomArena, cssom: &CssomArena, width: usize, height: usize) -> Self {
        let mut this = Self {
            arena: Arena::new(),
        };
        this.build_tree(dom);
        this.apply_css_style(cssom);

        this
    }

    fn apply_css_style(&mut self, cssom: &CssomArena) {
        self.apply_css_style_for(0, cssom);
    }

    fn apply_css_style_for(&mut self, id: NodeId, cssom: &CssomArena) {
        for rule in cssom.rules() {
            if cssom[*rule].selector().match_with(self, id) {
                self[id].css_style.apply_from(cssom[*rule].style());
            }
        }
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
                self.arena.push(Node::body(attributes.clone()));
                self.build_body(dom, dom_child_id, 0);
                break;
            }
        }
    }

    fn build_body(&mut self, dom: &DomArena, dom_parent_id: NodeId, arena_parent_id: NodeId) {
        let parent_css_style = self[arena_parent_id].css_style.inherit();

        for dom_child_id in dom.children(dom_parent_id) {
            match dom[dom_child_id].node_type {
                DomNodeType::Element {
                    ref name,
                    ref attributes,
                } => {
                    let render_node = RenderNode::new(
                        RenderNodeType::Element {
                            name: name.clone(),
                            attributes: attributes.clone(),
                        },
                        parent_css_style.clone(),
                    );
                    let arena_child_id = self.arena.insert_child(arena_parent_id, render_node);
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
                            RenderNode::new(RenderNodeType::Text(text), parent_css_style.clone()),
                        );
                    }
                }
                ref nt => {}
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

#[derive(Clone, Debug)]
pub struct RenderNode {
    node_type: NodeType,
    pub css_style: CssStyle,
}

impl RenderNode {
    pub fn new(node_type: NodeType, css_style: CssStyle) -> Self {
        Self {
            node_type,
            css_style,
        }
    }

    pub fn body(attributes: HashMap<String, String>) -> Self {
        Self {
            node_type: NodeType::Element {
                name: "body".into(),
                attributes,
            },
            css_style: CssStyle::default(),
        }
    }

    pub fn node_type(&self) -> &NodeType {
        &self.node_type
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

#[derive(Clone, Copy, Debug)]
pub struct RenderStyle {}

use super::AlphaColor;
use super::Color;
use super::Font;
use crate::arena::Arena;
use crate::arena::NodeId;
use crate::css::ComputedValue;
use crate::css::CssStyle;
use crate::css::CssomArena;
use crate::css::Display;
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

    pub fn new(dom: &DomArena, cssom: &CssomArena) -> Self {
        let mut this = Self {
            arena: Arena::new(),
        };
        this.build_tree(dom);
        this.apply_css_style(cssom);
        this.calc_render_style();

        this
    }

    fn calc_render_style(&mut self) {
        self.calc_render_style_for(0);
    }

    fn calc_render_style_for(&mut self, id: NodeId) {
        let css_style = &self[id].css_style;

        self[id].style = Some(RenderStyle {
            font_size: css_style.font_size.unwrap().compute(self, id).unwrap(),
            color: css_style.color.unwrap().compute().unwrap(),
            background_color: css_style.background_color.unwrap().compute().unwrap(),
            padding_top: css_style.padding_top.unwrap().compute(self, id).unwrap(),
            padding_right: css_style.padding_right.unwrap().compute(self, id).unwrap(),
            padding_bottom: css_style.padding_bottom.unwrap().compute(self, id).unwrap(),
            padding_left: css_style.padding_left.unwrap().compute(self, id).unwrap(),
            margin_top: css_style.margin_top.unwrap().compute(self, id),
            margin_right: css_style.margin_right.unwrap().compute(self, id),
            margin_bottom: css_style.margin_bottom.unwrap().compute(self, id),
            margin_left: css_style.margin_left.unwrap().compute(self, id),
            width: css_style.width.unwrap().compute(self, id),
            height: css_style.height.unwrap().compute(self, id),
        });

        let children: Vec<_> = self.children(id).collect();
        for child in children {
            self.calc_render_style_for(child);
        }
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
                        self[arena_parent_id].css_style.inherit_for_element(),
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
                            RenderNode::new(
                                RenderNodeType::Text(text),
                                self[arena_parent_id].css_style.inherit_for_text(),
                            ),
                        );
                    }
                }
                _ => {}
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
    pub node_type: NodeType,
    pub css_style: CssStyle,
    pub style: Option<RenderStyle>,
}

impl RenderNode {
    pub fn new(node_type: NodeType, css_style: CssStyle) -> Self {
        Self {
            node_type,
            css_style,
            style: None,
        }
    }

    pub fn body(attributes: HashMap<String, String>) -> Self {
        Self {
            node_type: NodeType::Element {
                name: "body".into(),
                attributes,
            },
            css_style: CssStyle::default(),
            style: None,
        }
    }

    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    pub fn style(&self) -> &RenderStyle {
        self.style.as_ref().expect("failed to get RenderNode.style")
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

impl RenderNodeType {
    pub fn is_replace_element(&self) -> bool {
        matches!(self, Self::Element{name, ..} if ["img"].contains(&name.as_str()))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RenderStyle {
    pub font_size: f32,
    pub color: AlphaColor,
    pub background_color: AlphaColor,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub margin_top: ComputedValue<f32>,
    pub margin_right: ComputedValue<f32>,
    pub margin_bottom: ComputedValue<f32>,
    pub margin_left: ComputedValue<f32>,
    pub width: ComputedValue<f32>,
    pub height: ComputedValue<f32>,
}

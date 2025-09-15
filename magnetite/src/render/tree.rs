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

    pub fn new(dom: &DomArena, cssom: &CssomArena, width: usize, height: usize) -> Self {
        let mut this = Self {
            arena: Arena::new(),
        };
        this.build_tree(dom, width, height);
        this.attach_style(cssom);

        this
    }

    fn inherit_style(&mut self, id: NodeId) {
        if let Some(parent_id) = self[id].parent() {
            let parent_style = &self[parent_id].style;
            self[id].style = RenderStyle {
                font_size: parent_style.font_size,
                color: parent_style.color,
                background_color: None,
                x: self[id].style.x,
                y: self[id].style.y,
                width: self[id].style.width,
                height: self[id].style.height,
                margin_top: 0.0,
                margin_right: 0.0,
                margin_bottom: 0.0,
                margin_left: 0.0,
            };
        }
    }

    fn attach_style(&mut self, cssom: &CssomArena) {
        self.attach_style_for(0, cssom, 0, 0);
    }

    fn attach_style_for(&mut self, id: NodeId, cssom: &CssomArena, x: isize, y: isize) {
        self.inherit_style(id);
        self.attach_cssom_style(id, cssom);
        self[id].style.x = Some(x + self[id].style.margin_left as isize);
        self[id].style.y = Some(y + self[id].style.margin_top as isize);
        let (width, height) = self.attach_style_for_children(id, cssom);
        self.attach_style_width_and_height(id, width, height);
    }

    fn attach_cssom_style(&mut self, id: NodeId, cssom: &CssomArena) {
        for rule_id in cssom.rules() {
            if cssom[*rule_id].selector().match_with(self, id) {
                let cssom_style = cssom[*rule_id].style();
                self[id].style.background_color = cssom_style
                    .background_color
                    .or(self[id].style.background_color);
                self[id].style.color = cssom_style.color.unwrap_or(self[id].style.color);
                self[id].style.font_size = cssom_style
                    .font_size
                    .as_ref()
                    .map(|value| value.as_pixel(self, id))
                    .unwrap_or(self[id].style.font_size);
                self[id].style.margin_top = cssom_style
                    .margin_top
                    .as_ref()
                    .map(|value| value.as_pixel(self, id))
                    .unwrap_or(self[id].style.margin_top);
                self[id].style.margin_right = cssom_style
                    .margin_right
                    .as_ref()
                    .map(|value| value.as_pixel(self, id))
                    .unwrap_or(self[id].style.margin_right);
                self[id].style.margin_bottom = cssom_style
                    .margin_bottom
                    .as_ref()
                    .map(|value| value.as_pixel(self, id))
                    .unwrap_or(self[id].style.margin_bottom);
                self[id].style.margin_left = cssom_style
                    .margin_left
                    .as_ref()
                    .map(|value| value.as_pixel(self, id))
                    .unwrap_or(self[id].style.margin_left);
                self[id].style.width = cssom_style
                    .width
                    .as_ref()
                    .map(|value| value.as_pixel(self, id) as usize)
                    .or(self[id].style.width);
                self[id].style.height = cssom_style
                    .height
                    .as_ref()
                    .map(|value| value.as_pixel(self, id) as usize)
                    .or(self[id].style.height);
            }
        }
    }

    fn attach_style_for_children(&mut self, id: NodeId, cssom: &CssomArena) -> (usize, usize) {
        match self[id].node_type {
            NodeType::Element { .. } => {
                let this_x = self[id].style.x();
                let this_y = self[id].style.y();
                let mut x = this_x;
                let mut y = this_y;

                let children: Vec<_> = self.children(id).collect();
                for child in children {
                    self.attach_style_for(child, cssom, this_x, y);
                    x += self[child].style.width() as isize
                        + self[child].style.margin_horz() as isize;
                    y += self[child].style.height() as isize
                        + self[child].style.margin_vert() as isize;
                }

                ((x - this_x) as usize, (y - this_y) as usize)
            }
            NodeType::Text(ref text) => {
                let style = &self.arena[id].style;
                let font = Font::default();
                let glyphs = font.glyph_str(&text, style.font_size());
                let Layout { width, height, .. } = font.layout_str(&glyphs);
                let style = &mut self.arena[id].style;
                let (width, height) = (width as usize, height as usize);
                style.width = Some(width);
                style.height = Some(height);

                (width, height)
            }
        }
    }

    fn attach_style_width_and_height(&mut self, id: NodeId, width: usize, height: usize) {
        let style = &mut self[id].style;
        style.width = style.width.or(Some(width));
        style.height = style.height.or(Some(height));
    }

    fn build_tree(&mut self, dom: &DomArena, width: usize, height: usize) {
        for dom_child_id in dom.children(DomArena::DOCUMENT_IDX) {
            if let DomNodeType::Element { ref name, .. } = dom[dom_child_id].node_type
                && name == "html"
            {
                self.build_html(dom, dom_child_id, width, height);
                break;
            }
        }
    }

    fn build_html(&mut self, dom: &DomArena, dom_parent_id: NodeId, width: usize, height: usize) {
        for dom_child_id in dom.children(dom_parent_id) {
            if let DomNodeType::Element {
                ref name,
                ref attributes,
            } = dom[dom_child_id].node_type
                && name == "body"
            {
                self.arena
                    .push(Node::body(attributes.clone(), width, height));
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

#[derive(Clone, Copy, Debug)]
pub struct RenderStyle {
    pub font_size: f32,
    pub color: Color,
    pub background_color: Option<Color>,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub x: Option<isize>,
    pub y: Option<isize>,
    pub width: Option<usize>,
    pub height: Option<usize>,
}

impl RenderStyle {
    pub fn new() -> Self {
        Self {
            font_size: 16.0,
            color: Color::BLACK,
            background_color: None,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            x: None,
            y: None,
            width: None,
            height: None,
        }
    }

    pub fn margin_horz(&self) -> f32 {
        self.margin_left + self.margin_right
    }

    pub fn margin_vert(&self) -> f32 {
        self.margin_top + self.margin_bottom
    }

    pub fn body(width: usize, height: usize) -> Self {
        Self {
            font_size: 16.0,
            color: Color::BLACK,
            background_color: None,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            x: Some(0),
            y: Some(0),
            width: Some(width),
            height: Some(height),
        }
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn color(&self) -> Color {
        self.color
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
    pub style: RenderStyle,
}

impl RenderNode {
    pub fn new(node_type: NodeType) -> Self {
        Self {
            node_type,
            style: RenderStyle::new(),
        }
    }

    pub fn body(attributes: HashMap<String, String>, width: usize, height: usize) -> Self {
        Self {
            node_type: NodeType::Element {
                name: "body".into(),
                attributes,
            },
            style: RenderStyle::body(width, height),
        }
    }

    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    pub fn style(&self) -> RenderStyle {
        self.style
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

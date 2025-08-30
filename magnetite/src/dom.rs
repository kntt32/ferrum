use std::collections::HashMap;
use std::iter::Iterator;
use std::ops::Index;
use std::ops::IndexMut;
use std::str::FromStr;

pub type DomNodeIdx = usize;
type NodeIdx = DomNodeIdx;
type Node = DomNode;
type NodeType = DomNodeType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomArena {
    arena: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomNode {
    namespace: Namespace,
    node_type: NodeType,
    parent: Option<NodeIdx>,
    child: Option<NodeIdx>,
    prev: Option<NodeIdx>,
    next: Option<NodeIdx>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Namespace {
    Html,
    MathMl,
    Svg,
    XLink,
    Xml,
    Xmlns,
}

impl FromStr for Namespace {
    type Err = ();

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if s.ends_with('/') {
            s = &s[..s.len() - 1];
        }
        if s.starts_with("http://") {
            s = &s[7..];
        }
        if s.starts_with("https://") {
            s = &s[8..];
        }
        if s.starts_with("www.") {
            s = &s[4..];
        }
        match s {
            "w3.org/1999/xhtml" => Ok(Self::Html),
            "w3.org/1998/Math/MathML" => Ok(Self::MathMl),
            "w3.org/2000/svg" => Ok(Self::Svg),
            "w3.org/1999/xlink" => Ok(Self::XLink),
            "w3.org/XML/1998/namespace" => Ok(Self::Xml),
            "w3.org/2000/xmlns" => Ok(Self::Xmlns),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomNodeType {
    Document,
    DocType(String),
    Element {
        name: String,
        attributes: HashMap<String, String>,
    },
    Comment(String),
    Text(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Siblings<'a> {
    dom_arena: &'a DomArena,
    node_index: NodeIdx,
}

impl DomArena {
    pub const DOCUMENT_IDX: usize = 0;

    pub fn new() -> Self {
        Self {
            arena: vec![Node::document()],
        }
    }

    pub fn get(&self, idx: NodeIdx) -> &DomNode {
        &self.arena[idx]
    }

    pub fn get_mut(&mut self, idx: NodeIdx) -> &mut DomNode {
        &mut self.arena[idx]
    }

    pub fn current_node(&self) -> NodeIdx {
        let len = self.arena.len();
        assert!(1 < len);
        len - 1
    }

    pub fn get_last_element(&self, name: &str) -> Option<NodeIdx> {
        for i in (0..self.arena.len()).rev() {
            if let NodeType::Element { name: ref n, .. } = self.arena[i].node_type
                && n == name
            {
                return Some(i);
            }
        }

        None
    }

    pub fn parent(&self, node: NodeIdx) -> Option<NodeIdx> {
        self.arena[node].parent
    }

    pub fn child(&self, node: NodeIdx) -> Option<NodeIdx> {
        self.arena[node].child
    }

    pub fn siblings(&self, node: NodeIdx) -> Siblings<'_> {
        Siblings {
            dom_arena: self,
            node_index: node,
        }
    }

    pub fn append_child(&mut self, to: NodeIdx, node: Node) {
        self.arena.push(node);
        let node_idx = self.current_node();

        if let Some(child_idx) = self.child(to) {
            let child_end_idx = self.siblings(child_idx).last().unwrap();
            self[node_idx].parent = Some(to);
            self[node_idx].prev = Some(child_end_idx);
            self[child_end_idx].next = Some(node_idx);
        } else {
            self[to].child = Some(node_idx);
        }
    }
}

impl Index<NodeIdx> for DomArena {
    type Output = Node;

    fn index(&self, index: NodeIdx) -> &Node {
        self.get(index)
    }
}

impl IndexMut<NodeIdx> for DomArena {
    fn index_mut(&mut self, index: NodeIdx) -> &mut Node {
        self.get_mut(index)
    }
}

impl DomNode {
    pub fn new(node_type: DomNodeType, namespace: Namespace) -> Self {
        Self {
            parent: None,
            child: None,
            namespace,
            node_type,
            prev: None,
            next: None,
        }
    }

    pub fn document() -> Self {
        Self {
            parent: None,
            child: None,
            namespace: Namespace::Html,
            node_type: NodeType::Document,
            prev: None,
            next: None,
        }
    }

    pub fn namespace(&self) -> Namespace {
        self.namespace
    }

    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }
}

impl<'a> Iterator for Siblings<'a> {
    type Item = NodeIdx;

    fn next(&mut self) -> Option<Self::Item> {
        let node_index = self.node_index;
        self.node_index = self.dom_arena.get(node_index).next?;
        Some(node_index)
    }
}

use crate::arena::Arena;
use crate::arena::ArenaNode;
pub use crate::arena::NodeId;
use std::collections::HashMap;
use std::iter::Iterator;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;
use std::slice::SliceIndex;
use std::str::FromStr;

type Node = DomNode;
type NodeType = DomNodeType;

#[derive(Clone, Debug)]
pub struct DomArena {
    arena: Arena<Node>,
}

impl DomArena {
    pub const DOCUMENT_IDX: usize = 0;

    pub fn new() -> Self {
        let mut arena = Arena::new();
        arena.push(Node::DOCUMENT);
        Self { arena }
    }

    pub fn get_child_element(&mut self, id: NodeId, name: &str) -> Option<NodeId> {
        for child in self.arena.children(id) {
            if let NodeType::Element {
                name: ref node_name,
                ..
            } = self[child].node_type
                && node_name == name
            {
                return Some(child);
            }
        }
        None
    }

    pub fn insert_child(&mut self, id: NodeId, mut node: Node) -> NodeId {
        if let Some(last_child) = self.arena.children(id).last() {
            self.insert_after(last_child, node)
        } else {
            if let NodeType::Character(c) = node.node_type {
                node.node_type = NodeType::String(format!("{}", c));
            }
            self.arena.insert_child(id, node)
        }
    }

    pub fn insert_after(&mut self, at: NodeId, mut node: Node) -> NodeId {
        if let NodeType::String(ref mut s) = self.arena[at].node_type
            && let NodeType::Character(c) = node.node_type
        {
            s.push(c);
            at
        } else if let NodeType::String(ref mut s) = self.arena[at].node_type
            && let NodeType::String(ref s2) = node.node_type
        {
            s.push_str(s2.as_str());
            at
        } else {
            if let NodeType::Character(c) = node.node_type {
                node.node_type = NodeType::String(format!("{}", c));
            }
            self.arena.insert_after(at, node)
        }
    }

    pub fn push(&mut self, mut node: Node) -> NodeId {
        if let NodeType::Character(c) = node.node_type {
            node.node_type = NodeType::String(format!("{}", c));
        }
        self.arena.push(node)
    }
}

impl<I: SliceIndex<[ArenaNode<Node>]>> Index<I> for DomArena {
    type Output = <I as SliceIndex<[ArenaNode<Node>]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.arena[index]
    }
}

impl<I: SliceIndex<[ArenaNode<Node>]>> IndexMut<I> for DomArena {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.arena[index]
    }
}

impl Deref for DomArena {
    type Target = Arena<Node>;

    fn deref(&self) -> &Self::Target {
        &self.arena
    }
}

impl DerefMut for DomArena {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.arena
    }
}

impl Default for DomArena {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomNode {
    namespace: Namespace,
    pub node_type: NodeType,
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
    Character(char),
    String(String),
}

impl DomNode {
    pub const DOCUMENT: Self = Self {
        namespace: Namespace::Html,
        node_type: NodeType::Document,
    };

    pub fn new(node_type: DomNodeType, namespace: Namespace) -> Self {
        Self {
            namespace,
            node_type,
        }
    }

    pub fn namespace(&self) -> Namespace {
        self.namespace
    }

    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }
}

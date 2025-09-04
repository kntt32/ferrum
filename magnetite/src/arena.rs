use ops::Deref;
use ops::DerefMut;
use ops::Index;
use ops::IndexMut;
use std::ops;
use std::slice::SliceIndex;

pub type NodeId = usize;

#[derive(Clone, Debug)]
pub struct Arena<T> {
    arena: Vec<ArenaNode<T>>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { arena: Vec::new() }
    }

    pub fn push(&mut self, value: T) -> NodeId {
        let node = ArenaNode {
            parent: None,
            child: None,
            next: None,
            prev: None,
            value,
        };
        self.arena.push(node);
        self.arena.len() - 1
    }

    pub fn unlink(&mut self, index: NodeId) {
        if let Some(prev) = self[index].prev {
            self[prev].next = self[index].next;
        }
        if let Some(next) = self[index].next {
            self[next].prev = self[index].prev;
        }
        self[index].parent = None;
        self[index].prev = None;
        self[index].next = None;
    }

    pub fn children(&self, id: NodeId) -> Siblings<'_, T> {
        Siblings {
            arena: self,
            id: self[id].child,
        }
    }

    pub fn siblings(&self, id: NodeId) -> Siblings<'_, T> {
        Siblings {
            arena: self,
            id: Some(id),
        }
    }

    pub fn insert_child(&mut self, id: NodeId, value: T) -> NodeId {
        if let Some(child) = self[id].child {
            self.append(child, value)
        } else {
            let child_id = self.push(value);
            self[id].child = Some(child_id);
            self[child_id].parent = Some(id);
            child_id
        }
    }

    pub fn append(&mut self, id: NodeId, value: T) -> NodeId {
        let last_sibling_id = self.siblings(id).last().unwrap();
        self.insert_after(last_sibling_id, value)
    }

    pub fn insert_after(&mut self, at: NodeId, value: T) -> NodeId {
        let id = self.push(value);
        self.insert_after_node(id, at);
        id
    }

    pub fn insert_after_node(&mut self, id: NodeId, at: NodeId) {
        self.unlink(id);

        self[id].next = self[at].next;
        self[id].prev = Some(at);
        self[id].parent = self[at].parent;

        if let Some(next) = self[at].next {
            self[next].prev = Some(id);
        }
        self[at].next = Some(id);
    }
}

impl<T, I> Index<I> for Arena<T>
where
    I: SliceIndex<[ArenaNode<T>]>,
{
    type Output = <I as SliceIndex<[ArenaNode<T>]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.arena[index]
    }
}

impl<T, I> IndexMut<I> for Arena<T>
where
    I: SliceIndex<[ArenaNode<T>]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.arena[index]
    }
}

#[derive(Clone, Debug)]
pub struct ArenaNode<T> {
    parent: Option<NodeId>,
    child: Option<NodeId>,
    next: Option<NodeId>,
    prev: Option<NodeId>,
    value: T,
}

impl<T> ArenaNode<T> {
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    pub fn child(&self) -> Option<NodeId> {
        self.child
    }

    pub fn next(&self) -> Option<NodeId> {
        self.next
    }

    pub fn prev(&self) -> Option<NodeId> {
        self.prev
    }
}

impl<T> Deref for ArenaNode<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for ArenaNode<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Siblings<'a, T> {
    arena: &'a Arena<T>,
    id: Option<NodeId>,
}

impl<'a, T> Iterator for Siblings<'a, T> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.id?;
        self.id = self.arena[id].next();
        Some(id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_arena() {
        let mut arena: Arena<i32> = Arena::new();
        let root = arena.push(1);
        let child1 = arena.insert_child(root, 2);
        let child2 = arena.insert_child(root, 3);
        arena.insert_child(root, 4);
        arena.insert_child(child2, 5);
        assert_eq!(
            &format!("{:?}", arena),
            "Arena { arena: [ArenaNode { parent: None, child: Some(1), next: None, prev: None, value: 1 }, ArenaNode { parent: None, child: None, next: Some(2), prev: None, value: 2 }, ArenaNode { parent: None, child: Some(4), next: Some(3), prev: Some(1), value: 3 }, ArenaNode { parent: None, child: None, next: None, prev: Some(2), value: 4 }, ArenaNode { parent: None, child: None, next: None, prev: None, value: 5 }] }"
        );
        arena.unlink(child2);
        assert_eq!(
            &format!("{:?}", arena),
            "Arena { arena: [ArenaNode { parent: None, child: Some(1), next: None, prev: None, value: 1 }, ArenaNode { parent: None, child: None, next: Some(3), prev: None, value: 2 }, ArenaNode { parent: None, child: Some(4), next: None, prev: None, value: 3 }, ArenaNode { parent: None, child: None, next: None, prev: Some(1), value: 4 }, ArenaNode { parent: None, child: None, next: None, prev: None, value: 5 }] }"
        );
        arena.unlink(child1);
        assert_eq!(
            &format!("{:?}", arena),
            "Arena { arena: [ArenaNode { parent: None, child: Some(1), next: None, prev: None, value: 1 }, ArenaNode { parent: None, child: None, next: None, prev: None, value: 2 }, ArenaNode { parent: None, child: Some(4), next: None, prev: None, value: 3 }, ArenaNode { parent: None, child: None, next: None, prev: None, value: 4 }, ArenaNode { parent: None, child: None, next: None, prev: None, value: 5 }] }"
        );
        println!("{:?}", arena);
    }
}

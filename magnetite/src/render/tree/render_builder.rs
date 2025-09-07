use super::RenderArena;
use crate::html::DomArena;
use crate::html::DomNodeType;
use crate::arena::NodeId;
use std::ops::ControlFlow;

type State = RenderArenaBuildState;
type Error = RenderArenaBuildError;

#[derive(Clone, Debug)]
pub struct RenderArenaBuilder<'a> {
    state: State,
    dom_cursor: NodeId,
    dom: &'a DomArena,
    arena_cursor: Option<NodeId>,
    arena: RenderArena,
    errors: Vec<Error>,
}

impl<'a> RenderArenaBuilder<'a> {
    pub fn new(dom: &'a DomArena) -> Self {
        Self {
            state: State::BeforeBody,
            dom_cursor: DomArena::DOCUMENT_IDX,
            dom,
            arena_cursor: None,
            arena: RenderArena::new(),
            errors: Vec::new(),
        }
    }

    fn switch_to(&mut self, state: State) {
        self.state = state;
    }

    fn error(&mut self, error: Error) {
        self.errors.push(error);
    }

    fn finish(&mut self) -> ControlFlow<&RenderArena> {
        self.state = State::Finish;
        ControlFlow::Break(&self.arena)
    }

    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    pub fn step(&mut self) -> ControlFlow<&RenderArena> {
        match self.state {
            State::BeforeBody => self.step_before_body(),
            State::Body => self.step_body(),
            State::AfterBody => self.step_after_body(),
            State::Finish => ControlFlow::Break(&self.arena),
        }
    }

    fn step_before_body(&mut self) -> ControlFlow<&RenderArena> {
        for child in self.dom.children(self.cursor) {
            if let DomNodeType::Element{ref name, ref attributes, ..} = self.dom[child].node_type && name == "body" {
                self.switch_to(State::Body);
                self.node_cursor = child;
                let render_node = RenderNode::body();
                self.arena_cursor = Some(self.arena.push(render_node));

                return ControlFlow::Continue(());
            }
        }

        self.error(Error::BodyNotFound);
        self.finish()
    }

    fn step_body(&mut self) -> ControlFlow<&RenderArena> {
        if let Some(ref dom_next) = self.dom[self.dom_cursor].next() {
            self.dom_cursor = dom_next;
            match self.dom[next] {
                DomNodeType::Element{ref name, ref attributes} => {
                    let parent_render_node = &self.arena[self.arena_cursor];
                    let 
                },
                DomNodeType::String(ref s) => {
                    let parent_arena = &self.arena[self.arena_cursor].parent().unwrap();
                    let parent_render_node = &self.arena[self.arena_cursor];
                    let render_node = parent_render_node.inherit(RenderNodeType::Text(s.clone()));
                    self.arena_cursor = self.arena.insert_after(self.arena_cursor, render_node);
                },
            }
            ControlFlow::Continue(())
        }else {
            if let Some(dom_parent) = self.dom[self.dom_cursor].parent() 
                && let Some(arena_parent) = self.arena[self.arena_cursor].parent() {
                self.dom_cursor = dom_parent;
                self.arena_cursor = arena_cursor;
                ControlFlow::Continue(())
            }else {
                self.finish()
            }
        }
    }

    fn step_after_body(&mut self) -> ControlFlow<&RenderArena> {
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
enum RenderArenaBuildState {
    BeforeBody,
    Body,
    AfterBody,
    Finish,
}

#[derive(Clone, Copy, Debug)]
enum RenderArenaBuildError {
    BodyNotFound,
}

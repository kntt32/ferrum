use super::RenderArena;

#[derive(Clone, Debug)]
pub struct Renderer {
    arena: RenderArena,
}

impl Renderer {
    pub fn new(arena: RenderArena) -> Self {
        Self { arena }
    }
}

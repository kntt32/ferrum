use super::Color;
use super::RenderArena;
use softbuffer::Buffer;

#[derive(Clone, Debug)]
pub struct Renderer {
    arena: RenderArena,
}

impl Renderer {
    pub fn new(arena: RenderArena) -> Self {
        Self { arena }
    }
}

pub trait Buff {
    fn draw_dot(&mut self, x: usize, y: usize, color: Color);
    fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: Color);
}

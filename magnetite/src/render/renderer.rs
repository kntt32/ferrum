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
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, x: usize, y: usize) -> Option<&u32>;
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut u32>;
}

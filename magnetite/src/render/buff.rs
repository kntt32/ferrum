use super::Color;

pub trait Buff {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, x: isize, y: isize) -> Option<&u32>;
    fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut u32>;

    fn draw_rect(&mut self, x: isize, y: isize, width: usize, height: usize, color: Color) {
        let code = color.as_u32();

        for yi in y..y + height as isize {
            for xi in x..x + width as isize {
                if let Some(b) = self.get_mut(xi, yi) {
                    *b = code;
                }
            }
        }
    }

    fn fill(&mut self, color: Color) {
        let width = self.width();
        let height = self.height();
        self.draw_rect(0, 0, width, height, color);
    }
}

pub struct SBuff<'a> {
    buff: &'a mut [u32],
    width: usize,
    height: usize,
}

impl<'a> SBuff<'a> {
    pub fn new(buff: &'a mut [u32], width: usize, height: usize) -> Self {
        Self {
            buff,
            width,
            height,
        }
    }
}

impl<'a> Buff for SBuff<'a> {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn get(&self, x: isize, y: isize) -> Option<&u32> {
        if 0 <= x && 0 <= y {
            self.buff.get(x as usize + y as usize * self.width)
        } else {
            None
        }
    }

    fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut u32> {
        if 0 <= x && 0 <= y {
            self.buff.get_mut(x as usize + y as usize * self.width)
        } else {
            None
        }
    }
}

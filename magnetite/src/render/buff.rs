use super::Color;

pub trait Drawer: Copy {
    fn draw(&self, b: &mut u32);
    fn draw_with_alpha(&self, b: &mut u32, alpha: f32);
}

pub trait Buff: Sized {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, x: isize, y: isize) -> Option<&u32>;
    fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut u32>;

    fn window(&mut self, x: isize, y: isize, width: usize, height: usize) -> impl Buff {
        Window::new(self, x, y, width, height)
    }

    fn draw_rect(&mut self, x: isize, y: isize, width: usize, height: usize, color: impl Drawer) {
        for yi in y..y + height as isize {
            for xi in x..x + width as isize {
                if let Some(b) = self.get_mut(xi, yi) {
                    color.draw(b);
                }
            }
        }
    }

    fn draw_rect_border(&mut self, x: isize, y: isize, width: usize, height: usize, color: Color) {
        let code = color.as_u32();
        let width = width as isize;
        let height = height as isize;

        for yi in y..y + height as isize {
            if let Some(b) = self.get_mut(x, yi) {
                *b = code;
            }
            if let Some(b) = self.get_mut(x + width, yi) {
                *b = code;
            }
        }
        for xi in x..x + width as isize {
            if let Some(b) = self.get_mut(xi, y) {
                *b = code;
            }
            if let Some(b) = self.get_mut(xi, y + height) {
                *b = code;
            }
        }
    }

    fn fill(&mut self, color: impl Drawer) {
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
        if 0 <= x && 0 <= y && x < self.width as isize && y < self.height as isize {
            self.buff.get(x as usize + y as usize * self.width)
        } else {
            None
        }
    }

    fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut u32> {
        if 0 <= x && 0 <= y && x < self.width as isize && y < self.height as isize {
            self.buff.get_mut(x as usize + y as usize * self.width)
        } else {
            None
        }
    }
}

pub struct Window<'a, T: Buff> {
    buff: &'a mut T,
    x: isize,
    y: isize,
    width: usize,
    height: usize,
}

impl<'a, T: Buff> Window<'a, T> {
    pub fn new(buff: &'a mut T, x: isize, y: isize, width: usize, height: usize) -> Self {
        Self {
            buff,
            x,
            y,
            width,
            height,
        }
    }
}

impl<'a, T: Buff> Buff for Window<'a, T> {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn get(&self, x: isize, y: isize) -> Option<&u32> {
        if x < self.width as isize && y < self.height as isize {
            self.buff.get(self.x + x, self.y + y)
        } else {
            None
        }
    }

    fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut u32> {
        if x < self.width as isize && y < self.height as isize {
            self.buff.get_mut(self.x + x, self.y + y)
        } else {
            None
        }
    }
}

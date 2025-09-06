use softbuffer::Context;
use softbuffer::Surface;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::dpi::Size;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::window::WindowId;

pub fn winit_and_softbuffer_demo() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}

pub struct App {
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    color: Color,
}

impl App {
    pub fn new() -> Self {
        Self {
            surface: None,
            color: Color::BLUE,
        }
    }

    fn draw(&mut self) {
        let surface = self.surface.as_mut().unwrap();
        let mut buffer = surface.buffer_mut().unwrap();
        buffer.fill(self.color.as_u32());
        buffer.present().unwrap();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        const SIZE: PhysicalSize<u32> = PhysicalSize {
            width: 400,
            height: 300,
        };

        let mut attributes = Window::default_attributes();
        attributes.inner_size = Some(Size::Physical(SIZE));

        let window = Rc::new(event_loop.create_window(attributes).unwrap());
        window.request_redraw();

        let context = Context::new(window.clone()).unwrap();
        let mut surface = Surface::new(&context, window).unwrap();
        surface.resize(NonZeroU32::new(300).unwrap(), NonZeroU32::new(400).unwrap()).unwrap();
        self.surface = Some(surface);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        println!("LOG: {:?}", event);
        match event {
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            WindowEvent::CloseRequested => {
                println!("LOG: exit");
                event_loop.exit();
            }
            WindowEvent::MouseInput { .. } => {
                self.color = self.color.rotate();
                println!("{:?}", self.color);
                self.draw();
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const WHITE: Self = Color {
        red: 0xff,
        green: 0xff,
        blue: 0xff,
    };
    pub const BLUE: Self = Color {
        red: 0x00,
        green: 0x00,
        blue: 0xff,
    };

    pub fn as_u32(&self) -> u32 {
        let red = self.red as u32;
        let green = self.green as u32;
        let blue = self.blue as u32;
        (red << 16) | (green << 8) | blue
    }

    pub fn rotate(self) -> Self {
        let Self { red, green, blue } = self;
        Self {
            red: blue,
            green: red,
            blue: green,
        }
    }
}

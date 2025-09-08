use magnetite::render::Buff;
use magnetite::render::Color;
use magnetite::render::Font;
use magnetite::render::SBuff;
use softbuffer::Context;
use softbuffer::Surface;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::LazyLock;
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

        let font = Font::default();
        let glyph = font.glyph('あ', 100.0);
        let mut buff = SBuff::new(&mut buffer, 400, 300);
        font.draw(glyph, &mut buff, 20, 30, Color::WHITE);

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
        surface
            .resize(NonZeroU32::new(400).unwrap(), NonZeroU32::new(300).unwrap())
            .unwrap();
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

use crate::render::Buff;
use crate::render::Color;
use crate::render::SBuff;
use softbuffer::Context;
use softbuffer::Surface;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::dpi::Size;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::window::WindowId;

pub fn render_demo() {
    let mut event_loop = EventLoop::new().unwrap();
    let mut app = DemoApp::new();
    event_loop.run_app(&mut app);
}

struct DemoApp {
    width: usize,
    height: usize,
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
}

impl DemoApp {
    pub fn new() -> Self {
        Self {
            width: 400,
            height: 300,
            window: None,
            surface: None,
        }
    }
}

impl ApplicationHandler for DemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes();
        attributes.inner_size = Some(Size::Physical(PhysicalSize {
            width: self.width as u32,
            height: self.height as u32,
        }));
        let mut window = event_loop.create_window(attributes).unwrap();
        window.set_title("magnetite demo");
        let window = Rc::new(window);
        self.window = Some(window.clone());

        let context = Context::new(window.clone()).unwrap();
        let mut surface = Surface::new(&context, window.clone()).unwrap();
        surface.resize(
            NonZeroU32::new(self.width as u32).unwrap(),
            NonZeroU32::new(self.height as u32).unwrap(),
        );
        self.surface = Some(surface);

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("LOG: exit");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let surface = self.surface.as_mut().unwrap();
                let mut buff = surface.buffer_mut().unwrap();
                let mut sbuff = SBuff::new(&mut buff, self.width, self.height);
                sbuff.fill(Color::GREEN);
                buff.present();
            }
            _ => {
                println!("LOG: {:?}", event);
            }
        }
    }
}

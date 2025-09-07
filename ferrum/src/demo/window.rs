use ab_glyph::Font;
use ab_glyph::FontRef;
use ab_glyph::PxScale;
use magnetite::render::Color;
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

static FONT: LazyLock<FontRef<'static>> = LazyLock::new(|| {
    let font = FontRef::try_from_slice(include_bytes!(
        "../../../assets/fonts/NotoSansJP-VariableFont_wght.ttf"
    ))
    .unwrap();
    font
});

fn render_font(c: char, width: usize, height: usize, buff: &mut [u32], color: u32) {
    let glyph = FONT.glyph_id(c).with_scale(PxScale { x: 100.0, y: 100.0 });
    FONT.outline_glyph(glyph).unwrap().draw(|x, y, alpha| {
        if 0.5 < alpha {
            println!("{},{},{}", x, y, alpha);
            buff[x as usize + y as usize * width] = color;
        }
    })
}

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
        render_font('A', 400, 300, &mut buffer, 0x00000000);
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

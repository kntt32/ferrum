use magnetite::css::CssomArena;
use magnetite::html::*;
use magnetite::render::*;
use softbuffer::Context;
use softbuffer::Surface;
use std::io::Cursor;
use std::io::Read;
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

pub fn view() {
    let stream = Cursor::new(
        r#"
<!DOCTYPE html>
<html><head>
    <title>Example Domain</title>

    <meta charset="utf-8">
    <meta http-equiv="Content-type" content="text/html; charset=UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style type="text/css">
    body {
        background-color: #f0f0f2;
        margin: 0;
        padding: 0;
        font-family: -apple-system, system-ui, BlinkMacSystemFont, "Segoe UI", "Open Sans", "Helvetica Neue", Helvetica, Arial, sans-serif;
        
    }
    div {
        width: 600px;
        margin: 5em auto;
        padding: 2em;
        background-color: #fdfdff;
        border-radius: 0.5em;
        box-shadow: 2px 3px 7px 2px rgba(0,0,0,0.02);
    }
    a:link, a:visited {
        color: #38488f;
        text-decoration: none;
    }
    @media (max-width: 700px) {
        div {
            margin: 0 auto;
            width: auto;
        }
    }
    </style>    
</head>

<body>
<div>
    <h1>Example Domain</h1>
    <p>This domain is for use in illustrative examples in documents. You may use this
    domain in literature without prior coordination or asking for permission.</p>
    <p><a href="https://www.iana.org/domains/example">More information...</a></p>
</div>
</body></html>
"#,
    ); /*
    let stream = Cursor::new(
    r#"
    <!DOCTYPE html>
    <html>
    <head>
    <style>
    h1 {
    color: blue;
    }
    </style>
    </head>
    <body>
    <h1>
    Hello,
    </h1>
    <p>World!</p>
    </body>
    </html>"#,
    );*/

    let mut app = Ferrum::new(
        stream,
        NonZeroU32::new(800).unwrap(),
        NonZeroU32::new(600).unwrap(),
    );
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app).unwrap();
}

pub struct Ferrum {
    width: NonZeroU32,
    height: NonZeroU32,
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    dom: DomArena,
    cssom: CssomArena,
    renderer: Renderer,
}

impl Ferrum {
    pub fn new(stream: impl Read, width: NonZeroU32, height: NonZeroU32) -> Self {
        let byte_stream_decoder = ByteStreamDecoder::new(stream);
        let input_stream_preprocessor = InputStreamPreprocessor::new(byte_stream_decoder).unwrap();
        let mut tree_constructor = TreeConstructor::new();
        let mut tokenizer = Tokenizer::new(input_stream_preprocessor, &mut tree_constructor);
        loop {
            if tokenizer.step().is_none() {
                break;
            }
        }
        let dom = tree_constructor.take_dom();
        let cssom = dom.cssom();
        let render_arena = RenderArena::new(&dom, &cssom);
        println!("{}", *render_arena);
        let layout_arena = LayoutArena::new(&render_arena, width.get() as f32);
        println!("{}", *layout_arena);
        let renderer = Renderer::new(render_arena, layout_arena);

        Self {
            width,
            height,
            window: None,
            surface: None,
            dom,
            renderer,
            cssom,
        }
    }

    fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) {
        self.width = width;
        self.height = height;

        let render_arena = RenderArena::new(&self.dom, &self.cssom);
        let layout_arena = LayoutArena::new(&render_arena, width.get() as f32);
        self.renderer = Renderer::new(render_arena, layout_arena);

        let window = self.window.as_ref().unwrap().clone();
        let context = Context::new(window.clone()).unwrap();
        let mut surface = Surface::new(&context, window.clone()).unwrap();
        surface.resize(self.width, self.height).unwrap();
        self.surface = Some(surface);

        window.request_redraw();
    }
}

impl ApplicationHandler for Ferrum {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes();
        attributes.inner_size = Some(Size::Physical(PhysicalSize {
            width: self.width.get(),
            height: self.height.get(),
        }));
        let window = event_loop.create_window(attributes).unwrap();
        window.set_title("ferrum");
        let window = Rc::new(window);
        self.window = Some(window.clone());

        self.resize(self.width, self.height);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                self.resize(
                    NonZeroU32::new(width).unwrap(),
                    NonZeroU32::new(height).unwrap(),
                );
            }
            WindowEvent::RedrawRequested => {
                let surface = self.surface.as_mut().unwrap();
                let mut buff = surface.buffer_mut().unwrap();
                let mut sbuff = SBuff::new(
                    &mut buff,
                    self.width.get() as usize,
                    self.height.get() as usize,
                );
                self.renderer.render(&mut sbuff);
                buff.present().unwrap();
            }
            _ => {}
        }
    }
}

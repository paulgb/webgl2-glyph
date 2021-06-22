use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use webgl2_glyph::{FontArc, TextRenderer, Section, Text};
use web_sys::WebGl2RenderingContext;

#[allow(unused)]
macro_rules! console_log {
    ($($x: expr), +) => (
        web_sys::console::log_1(&wasm_bindgen::JsValue::from(
            format!($($x),+)));
    )
}

struct Animation {
    _gl: &'static WebGl2RenderingContext,
    renderer: TextRenderer<'static>,
    frame: u32,
}

impl Animation {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        let gl = Box::leak(Box::new(gl));
        let font =
            FontArc::try_from_slice(include_bytes!("../../SourceSansPro-Regular.ttf")).unwrap();

        let renderer = TextRenderer::new(gl, font);

        Animation {
            _gl: gl,
            renderer,
            frame: 0,
        }
    }

    pub fn render(&mut self) {
        let size: f32 = (self.frame % 200) as f32 + 1.;
        self.renderer.glyph_brush().queue(
            Section::default()
                .add_text(Text::new("Hello world").with_scale(size))
                .with_screen_position((30., 30.)),
        );

        self.renderer.render();

        self.frame += 1;
    }
}

pub fn main() {
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let window = web_sys::window().unwrap();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let gl = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>().unwrap();

    let mut animation = Animation::new(gl);

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        animation.render();

        let window = web_sys::window().unwrap();
        window.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }) as Box<dyn FnMut()>));

    window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
}
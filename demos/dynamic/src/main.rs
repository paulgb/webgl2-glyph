use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;
use webgl2_glyph::{
    glyph_brush::{FontArc, Section, Text},
    TextRenderer,
};

#[allow(unused)]
macro_rules! console_log {
    ($($x: expr), +) => (
        web_sys::console::log_1(&wasm_bindgen::JsValue::from(
            format!($($x),+)));
    )
}

struct Animation {
    renderer: TextRenderer,
    frame: u32,
}

impl Animation {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        let font =
            FontArc::try_from_slice(include_bytes!("../../SourceSansPro-Regular.ttf")).unwrap();

        let renderer = TextRenderer::try_new(Rc::new(gl), font).unwrap();

        Animation {
            renderer,
            frame: 0,
        }
    }

    pub fn render(&mut self) {
        self.renderer.glyph_brush().queue(
            Section::default()
                .add_text(Text::new(&format!("Frame: {:?}", self.frame)).with_scale(50.))
                .with_screen_position((30., 30.)),
        );

        self.renderer.render().unwrap();

        self.frame += 1;
    }
}

pub fn main() {
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let window = web_sys::window().unwrap();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement =
        canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let gl = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()
        .unwrap();

    let mut animation = Animation::new(gl);

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        animation.render();

        let window = web_sys::window().unwrap();
        window
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}

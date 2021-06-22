use webgl2_glyph::{FontArc, TextRenderer, Section, Text};
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    console_error_panic_hook::set_once();

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

    let font =
        FontArc::try_from_slice(include_bytes!("../../SourceSansPro-Regular.ttf")).unwrap();

    let mut renderer = TextRenderer::new(&gl, font);

    renderer.glyph_brush().queue(
        Section::default()
            .add_text(Text::new("Hello world").with_scale(50.))
            .with_screen_position((30., 30.)),
    );

    renderer.render();

    Ok(())
}

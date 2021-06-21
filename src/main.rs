use glyph_brush::ab_glyph::point;
use glyph_brush::ab_glyph::FontArc;
use glyph_brush::ab_glyph::Rect;
use glyph_brush::GlyphBrush;
use glyph_brush::{BrushAction, GlyphBrushBuilder, GlyphVertex, Rectangle, Section, Text};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGlProgram;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlTexture};

use crate::shader::compile_shader;
use crate::shader::link_program;

mod error;
mod shader;

#[allow(unused)]
macro_rules! console_log {
    ($($x: expr), +) => (
        web_sys::console::log_1(&wasm_bindgen::JsValue::from(
            format!($($x),+)));
    )
}

#[rustfmt::skip]
pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> [f32; 16] {
    let tx = -(right + left) / (right - left);
    let ty = (top + bottom) / (top - bottom);
    let tz = -(far + near) / (far - near);
    [
        2.0 / (right - left), 0.0, 0.0, 0.0,
        0.0, 2.0 / (top - bottom), 0.0, 0.0,
        0.0, 0.0, 1., 0.0,
        tx, ty, tz, 1.0,
    ]
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Clone, Copy)]
struct VertexData {
    pos: [f32; 3],
    tex_pos: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Clone, Copy)]
struct TriangleData([VertexData; 3]);

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Clone, Copy)]
struct QuadData([TriangleData; 2]);

#[inline]
fn to_quad_data(vertex: GlyphVertex) -> QuadData {
    console_log!("vtx: {:?}", &vertex);

    let GlyphVertex {
        mut tex_coords,
        pixel_coords,
        extra,
        bounds,
        ..
    } = vertex;

    let gl_bounds = bounds;

    let mut gl_rect = Rect {
        min: point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
        max: point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
    };

    // handle overlapping bounds, modify uv_rect to preserve texture aspect
    if gl_rect.max.x > gl_bounds.max.x {
        let old_width = gl_rect.width();
        gl_rect.max.x = gl_bounds.max.x;
        tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.min.x < gl_bounds.min.x {
        let old_width = gl_rect.width();
        gl_rect.min.x = gl_bounds.min.x;
        tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.max.y > gl_bounds.max.y {
        let old_height = gl_rect.height();
        gl_rect.max.y = gl_bounds.max.y;
        tex_coords.max.y = tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
    }
    if gl_rect.min.y < gl_bounds.min.y {
        let old_height = gl_rect.height();
        gl_rect.min.y = gl_bounds.min.y;
        tex_coords.min.y = tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
    }

    QuadData([
        TriangleData([
            VertexData {
                pos: [pixel_coords.min.x, -pixel_coords.min.y, extra.z],
                tex_pos: [tex_coords.min.x, tex_coords.min.y],
                color: extra.color,
            },
            VertexData {
                pos: [pixel_coords.min.x, -pixel_coords.max.y, extra.z],
                tex_pos: [tex_coords.min.x, tex_coords.max.y],
                color: extra.color,
            },
            VertexData {
                pos: [pixel_coords.max.x, -pixel_coords.min.y, extra.z],
                tex_pos: [tex_coords.max.x, tex_coords.min.y],
                color: extra.color,
            },
        ]),
        TriangleData([
            VertexData {
                pos: [pixel_coords.min.x, -pixel_coords.max.y, extra.z],
                tex_pos: [tex_coords.min.x, tex_coords.max.y],
                color: extra.color,
            },
            VertexData {
                pos: [pixel_coords.max.x, -pixel_coords.min.y, extra.z],
                tex_pos: [tex_coords.max.x, tex_coords.min.y],
                color: extra.color,
            },
            VertexData {
                pos: [pixel_coords.max.x, -pixel_coords.max.y, extra.z],
                tex_pos: [tex_coords.max.x, tex_coords.max.y],
                color: extra.color,
            },
        ]),
    ])
}

#[inline]
fn describe_attribute(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    attribute: &str,
    offset: i32,
    size: i32,
    stride: usize,
) -> i32 {
    let location = gl.get_attrib_location(&program, attribute) as u32;
    gl.vertex_attrib_pointer_with_i32(
        location,
        size,
        WebGl2RenderingContext::FLOAT,
        false,
        stride as _,
        offset,
    );
    gl.enable_vertex_attrib_array(location);

    offset + size * std::mem::size_of::<f32>() as i32
}

struct TextRenderer<'a> {
    gl: &'a WebGl2RenderingContext,
    glyph_brush: GlyphBrush<QuadData>,
    program: WebGlProgram,
    texture: WebGlTexture,
}

impl<'a> TextRenderer<'a> {
    pub fn new(gl: &'a WebGl2RenderingContext) -> Self {
        let mut glyph_brush: GlyphBrush<QuadData> = {
            let font =
                FontArc::try_from_slice(include_bytes!("../SourceSansPro-Regular.ttf")).unwrap();
            GlyphBrushBuilder::using_font(font).build()
        };

        let texture = gl.create_texture().unwrap();

        {
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

            let dimensions = glyph_brush.texture_dimensions();

            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D, // target
                0,                                  // level
                WebGl2RenderingContext::R8 as _,    // internalformat
                dimensions.0 as _,
                dimensions.1 as _,
                0,
                WebGl2RenderingContext::RED,           // format
                WebGl2RenderingContext::UNSIGNED_BYTE, // type
                None,
            )
            .unwrap();

            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_S,
                WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_T,
                WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
        }

        glyph_brush.queue(
            Section::default()
                .add_text(Text::new("Hello world").with_scale(50.))
                .with_screen_position((30., 30.)),
        );

        let program = {
            let vert_shader = compile_shader(
                &gl,
                WebGl2RenderingContext::VERTEX_SHADER,
                include_str!("shader.vert"),
            )
            .unwrap();
            let frag_shader = compile_shader(
                &gl,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("shader.frag"),
            )
            .unwrap();
            link_program(&gl, &vert_shader, &frag_shader).unwrap()
        };

        TextRenderer {
            gl,
            glyph_brush,
            program,
            texture,
        }
    }

    pub fn render(&mut self) {
        let gl = &self.gl;

        let update_texture = move |rect: Rectangle<u32>, tex_data: &[u8]| {
            gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D, // target
                0,                                  // level
                0,                                  // xoffset
                0,                                  // yoffset
                rect.width() as _,
                rect.height() as _,
                WebGl2RenderingContext::RED,           // format
                WebGl2RenderingContext::UNSIGNED_BYTE, // type
                Some(&tex_data),
            )
            .unwrap();
        };

        match self
            .glyph_brush
            .process_queued(update_texture, to_quad_data)
        {
            Ok(BrushAction::Draw(vertices)) => {
                let vertex_buffer = self.gl.create_buffer().unwrap();
                self.gl
                    .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
                unsafe {
                    let quad_array = js_sys::Float32Array::view(&bytemuck::cast_slice(&vertices));

                    self.gl.buffer_data_with_array_buffer_view(
                        WebGl2RenderingContext::ARRAY_BUFFER,
                        &quad_array,
                        WebGl2RenderingContext::STATIC_DRAW,
                    );
                }

                let mut offset = 0;
                offset = describe_attribute(
                    &self.gl,
                    &self.program,
                    "a_position",
                    offset,
                    3,
                    std::mem::size_of::<VertexData>(),
                );
                offset = describe_attribute(
                    &self.gl,
                    &self.program,
                    "a_tex_coord",
                    offset,
                    2,
                    std::mem::size_of::<VertexData>(),
                );
                offset = describe_attribute(
                    &self.gl,
                    &self.program,
                    "a_color",
                    offset,
                    4,
                    std::mem::size_of::<VertexData>(),
                );

                self.gl.use_program(Some(&self.program));

                {
                    let canvas = gl
                        .canvas()
                        .unwrap()
                        .dyn_into::<web_sys::HtmlCanvasElement>()
                        .unwrap();
                    let width = canvas.width();
                    let height = canvas.height();
                    let transform = ortho(0., width as _, 0., height as _, 0., 1.);
                    let location = gl.get_uniform_location(&self.program, "u_transform");

                    gl.uniform_matrix4fv_with_f32_array(
                        location.as_ref(),
                        false,
                        &bytemuck::cast_slice(&transform),
                    );
                }

                self.gl.draw_arrays(
                    WebGl2RenderingContext::TRIANGLES,
                    0,
                    (vertices.len() * 6) as _,
                );
            }
            Ok(BrushAction::ReDraw) => {}
            Err(glyph_brush::BrushError::TextureTooSmall { suggested }) => {}
        }
    }
}

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

    let mut renderer = TextRenderer::new(&gl);
    renderer.render();

    Ok(())
}

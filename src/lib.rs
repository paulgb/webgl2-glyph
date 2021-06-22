pub use glyph_brush::{BrushAction, GlyphBrush, GlyphBrushBuilder, Rectangle, Section, Text};
pub use glyph_brush::ab_glyph::FontArc;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlTexture, WebGlBuffer};

use crate::projection::ortho;
use crate::shader::{compile_shader, link_program};
use crate::vertex::{QuadData, VertexData};
use wasm_bindgen::JsCast;

#[allow(unused)]
macro_rules! console_log {
    ($($x: expr), +) => (
        web_sys::console::log_1(&wasm_bindgen::JsValue::from(
            format!($($x),+)));
    )
}

mod error;
mod projection;
mod shader;
mod vertex;

pub struct TextRenderer<'a> {
    gl: &'a WebGl2RenderingContext,
    glyph_brush: GlyphBrush<QuadData>,
    program: WebGlProgram,
    vertex_buffer: WebGlBuffer,
    texture: WebGlTexture,
}

impl<'a> TextRenderer<'a> {
    pub fn glyph_brush(&mut self) -> &mut GlyphBrush<QuadData> {
        &mut self.glyph_brush
    }

    pub fn new(gl: &'a WebGl2RenderingContext, font: FontArc) -> Self {
        let glyph_brush: GlyphBrush<QuadData> = {
            GlyphBrushBuilder::using_font(font).build()
        };

        let texture = gl.create_texture().unwrap();
        let vertex_buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.buffer_data_with_i32(WebGl2RenderingContext::ARRAY_BUFFER, 4096, WebGl2RenderingContext::DYNAMIC_DRAW);

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

            gl.pixel_storei(WebGl2RenderingContext::UNPACK_ALIGNMENT, 1);
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
            vertex_buffer,
            texture,
        }
    }

    pub fn render(&mut self) {
        let gl = &self.gl;
        let texture = &self.texture;

        let update_texture = move |rect: Rectangle<u32>, tex_data: &[u8]| {
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));

            gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D, // target
                0,                                  // level
                rect.min[0] as _,                   // xoffset
                rect.min[1] as _,                   // yoffset
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
            .process_queued(update_texture, vertex::to_quad_data)
        {
            Ok(BrushAction::Draw(vertices)) => {
                self.gl
                    .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vertex_buffer));

                self.gl.buffer_sub_data_with_i32_and_u8_array(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    0,
                    &bytemuck::cast_slice(&vertices),
                );

                let mut offset = 0;
                offset = vertex::describe_attribute(
                    &self.gl,
                    &self.program,
                    "a_position",
                    offset,
                    3,
                    std::mem::size_of::<VertexData>(),
                );
                offset = vertex::describe_attribute(
                    &self.gl,
                    &self.program,
                    "a_tex_coord",
                    offset,
                    2,
                    std::mem::size_of::<VertexData>(),
                );
                vertex::describe_attribute(
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
            Ok(BrushAction::ReDraw) => {

            }
            Err(glyph_brush::BrushError::TextureTooSmall { suggested }) => {
                console_log!("Resize suggested {:?}", &suggested);
            }
        }
    }
}

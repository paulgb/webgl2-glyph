use ::glyph_brush::ab_glyph::FontArc;
use ::glyph_brush::{BrushAction, GlyphBrush, GlyphBrushBuilder, Rectangle};
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlTexture, WebGlUniformLocation};

use crate::error::WebGl2GlyphError;
pub use crate::fps::FpsCounter;
use crate::projection::ortho;
use crate::shader::{compile_shader, link_program};
use crate::vertex::{QuadData, VertexData};
use std::error::Error;
use std::rc::Rc;
use wasm_bindgen::JsCast;

#[allow(unused)]
macro_rules! console_log {
    ($($x: expr), +) => (
        web_sys::console::log_1(&wasm_bindgen::JsValue::from(
            format!($($x),+)));
    )
}

mod error;
mod fps;
mod projection;
mod shader;
mod vertex;

/// Re-exported glyph_brush.
pub mod glyph_brush {
    pub use ::glyph_brush::ab_glyph::FontArc;
    pub use ::glyph_brush::*;
}

/// Glyph renderer for WebGL2.
///
/// Example usage:
///
/// ```
/// use wasm_bindgen::JsCast;
/// use web_sys::WebGl2RenderingContext;
/// use webgl2_glyph::{TextRenderer, glyph_brush::{FontArc, Section, Text}};
///
/// let document = web_sys::window().unwrap().document().unwrap();
/// let canvas = document.get_element_by_id("canvas").unwrap();
/// let canvas: web_sys::HtmlCanvasElement =
/// canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
///
/// let gl = canvas
///     .get_context("webgl2")
///     .unwrap()
///     .unwrap()
///     .dyn_into::<WebGl2RenderingContext>()
///     .unwrap();
///
/// let font =
///     FontArc::try_from_slice(include_bytes!("../demos/SourceSansPro-Regular.ttf")).unwrap();
/// let mut renderer = TextRenderer::try_new(&gl, font).unwrap();
///
/// renderer.glyph_brush().queue(
///     Section::default()
///         .add_text(Text::new("Hello world").with_scale(50.))
///         .with_screen_position((30., 30.)),
/// );
///
/// renderer.render().unwrap();
/// ```
pub struct TextRenderer {
    gl: Rc<WebGl2RenderingContext>,
    glyph_brush: GlyphBrush<QuadData>,
    program: WebGlProgram,
    vertex_buffer: ReusableBuffer,
    texture: WebGlTexture,

    height: f32,
    width: f32,

    position_location: u32,
    tex_coord_location: u32,
    color_location: u32,
    uniform_location: WebGlUniformLocation,

    vertices: i32,

    pub x_offset: f32,
    pub y_offset: f32,
}

struct ReusableBuffer {
    buf: WebGlBuffer,
    gl: Rc<WebGl2RenderingContext>,
    size: i32,
}

impl ReusableBuffer {
    pub fn new(gl: Rc<WebGl2RenderingContext>) -> Result<Self, WebGl2GlyphError> {
        let size = 1024;

        let buf = gl
            .create_buffer()
            .ok_or_else(|| WebGl2GlyphError::WebGlError("Couldn't create buffer.".to_string()))?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buf));
        gl.buffer_data_with_i32(
            WebGl2RenderingContext::ARRAY_BUFFER,
            size,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        Ok(Self { buf, gl, size })
    }

    pub fn bind(&self) {
        self.gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.buf));
    }

    pub fn set_content(&mut self, content: &[u8]) -> Result<(), WebGl2GlyphError> {
        console_log!("set_content called");
        if content.len() as i32 > self.size {
            self.gl.delete_buffer(Some(&self.buf));

            self.buf = self.gl.create_buffer().ok_or_else(|| {
                WebGl2GlyphError::WebGlError("Couldn't create buffer.".to_string())
            })?;

            console_log!("Resizing buffer from {} to {}", self.size, content.len());
            self.size = content.len() as _;

            self.gl
                .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.buf));

            self.gl.buffer_data_with_i32(
                WebGl2RenderingContext::ARRAY_BUFFER,
                self.size,
                WebGl2RenderingContext::DYNAMIC_DRAW,
            );
        } else {
            self.gl
                .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.buf));
        }

        self.gl.buffer_sub_data_with_i32_and_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            0,
            content,
        );

        Ok(())
    }
}

impl TextRenderer {
    /// Returns a mutable reference to the renderer's internal `GlyphBrush` instance.
    /// This can be used to add text to the queue.
    pub fn glyph_brush(&mut self) -> &mut GlyphBrush<QuadData> {
        &mut self.glyph_brush
    }

    fn create_texture(
        gl: &WebGl2RenderingContext,
        dimensions: (u32, u32),
    ) -> Result<WebGlTexture, Box<dyn Error>> {
        let texture = gl
            .create_texture()
            .ok_or_else(|| WebGl2GlyphError::WebGlError("Could not create texture".to_string()))?;
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
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
        .map_err(|_| WebGl2GlyphError::WebGlError("Could not load into texture.".to_string()))?;

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

        Ok(texture)
    }

    /// Construct a new instance for rendering text in the given font to the given WebGL2 rendering
    /// context.
    pub fn try_new(gl: Rc<WebGl2RenderingContext>, font: FontArc) -> Result<Self, Box<dyn Error>> {
        let glyph_brush: GlyphBrush<QuadData> = { GlyphBrushBuilder::using_font(font).build() };
        let vertex_buffer = ReusableBuffer::new(gl.clone())?;

        let texture = Self::create_texture(&gl, glyph_brush.texture_dimensions())?;

        let program = {
            let vert_shader = compile_shader(
                &gl,
                WebGl2RenderingContext::VERTEX_SHADER,
                include_str!("shader.vert"),
            )?;
            let frag_shader = compile_shader(
                &gl,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("shader.frag"),
            )?;
            link_program(&gl, &vert_shader, &frag_shader)?
        };

        let canvas = gl
            .canvas()
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let width = canvas.width();
        let height = canvas.height();

        let position_location = gl.get_attrib_location(&program, "a_position") as u32;
        let tex_coord_location = gl.get_attrib_location(&program, "a_tex_coord") as u32;
        let color_location = gl.get_attrib_location(&program, "a_color") as u32;
        let uniform_location = gl.get_uniform_location(&program, "u_transform").unwrap();

        Ok(TextRenderer {
            gl,
            glyph_brush,
            program,
            vertex_buffer,
            texture,

            position_location,
            tex_coord_location,
            color_location,
            uniform_location,

            height: height as _,
            width: width as _,

            vertices: 0,

            x_offset: 0.,
            y_offset: 0.,
        })
    }

    /// Render the queued text. Should be called from a `request_animation_frame` callback.
    /// Rendering clears the draw queue.
    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let gl = &self.gl;

            gl.enable(WebGl2RenderingContext::BLEND);
            gl.blend_func(
                WebGl2RenderingContext::ONE,
                WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            );

            let texture = &self.texture;

            let update_texture = move |rect: Rectangle<u32>, tex_data: &[u8]| {
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

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
                    self.vertices = vertices.len() as _;
                    self.vertex_buffer
                        .set_content(&bytemuck::cast_slice(&vertices))?;

                    self.draw();
                    break;
                }
                Ok(BrushAction::ReDraw) => {
                    self.vertex_buffer.bind();
                    self.draw();
                    break;
                }
                Err(glyph_brush::BrushError::TextureTooSmall { suggested }) => {
                    self.texture = Self::create_texture(gl, suggested)?;
                    self.glyph_brush.resize_texture(suggested.0, suggested.1);
                }
            }
        }
        Ok(())
    }

    fn draw(&self) {
        self.gl.use_program(Some(&self.program));

        let mut offset = 0;
        offset = vertex::describe_attribute(
            &self.gl,
            self.position_location,
            offset,
            3, // vec3(x, y, z)
            std::mem::size_of::<VertexData>(),
        );
        offset = vertex::describe_attribute(
            &self.gl,
            self.tex_coord_location,
            offset,
            2, // vec2(u, v)
            std::mem::size_of::<VertexData>(),
        );
        vertex::describe_attribute(
            &self.gl,
            self.color_location,
            offset,
            4, // vec2(r, g, b, a)
            std::mem::size_of::<VertexData>(),
        );

        {
            let transform = ortho(
                -self.x_offset,
                -self.x_offset + self.width,
                -self.y_offset,
                -self.y_offset + self.height,
                0.,
                1.,
            );

            self.gl.uniform_matrix4fv_with_f32_array(
                Some(&self.uniform_location),
                false,
                &bytemuck::cast_slice(&transform),
            );
        }

        self.gl
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, self.vertices * 6);
    }
}

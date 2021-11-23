use glyph_brush::ab_glyph::{point, Rect};
use glyph_brush::GlyphVertex;
use web_sys::WebGl2RenderingContext;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Clone, Copy)]
pub struct VertexData {
    pos: [f32; 3],
    tex_pos: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Clone, Copy)]
struct TriangleData([VertexData; 3]);

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Clone, Copy)]
pub struct QuadData([TriangleData; 2]);

#[inline]
pub fn to_quad_data(vertex: GlyphVertex) -> QuadData {
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
pub fn describe_attribute(
    gl: &WebGl2RenderingContext,
    location: u32,
    offset: i32,
    size: i32,
    stride: usize,
) -> i32 {
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

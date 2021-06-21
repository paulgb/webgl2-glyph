use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

use crate::error::GlyphAtlasError;

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, GlyphAtlasError> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| GlyphAtlasError::WebGlError("Error creating shader.".to_string()))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .map(|d| GlyphAtlasError::WebGlShaderInfoLog(d))
            .unwrap_or_else(|| GlyphAtlasError::WebGlError("Error compiling shader.".to_string())))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, GlyphAtlasError> {
    let program = context
        .create_program()
        .ok_or_else(|| GlyphAtlasError::WebGlError("Error creating program.".to_string()))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .map(|d| GlyphAtlasError::WebGlProgramInfoLog(d))
            .unwrap_or_else(|| GlyphAtlasError::WebGlError("Error linking program.".to_string())))
    }
}

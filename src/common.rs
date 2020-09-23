use crate::error::{CmcError, CmcResult};
use web_sys::WebGlRenderingContext as WebGL;
use web_sys::*;

pub fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> CmcResult<WebGlShader> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or(CmcError::missing_val("Create shader"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    let status = gl.get_shader_parameter(&shader, WebGL::COMPILE_STATUS).as_bool().unwrap_or(false);

    if status {
        Ok(shader)
    } else {
        let log = gl.get_shader_info_log(&shader).ok_or(CmcError::missing_val("Shader info log"))?;
        Err(CmcError::ShaderCompile { log })
    }
}

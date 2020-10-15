use crate::error::{CmcError, CmcResult};
use web_sys::WebGlRenderingContext as WebGL;
use web_sys::*;
use wavefront_obj::obj::Vertex;

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

pub fn build_program(gl: &WebGlRenderingContext, vert_shader: &str, frag_shader: &str) -> CmcResult<WebGlProgram> {
    let program = gl.create_program().ok_or(CmcError::missing_val("create program"))?;
    let vert_shader = compile_shader(&gl, WebGL::VERTEX_SHADER, vert_shader)?;
    let frag_shader = compile_shader(&gl, WebGL::FRAGMENT_SHADER, frag_shader)?;

    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);
    gl.link_program(&program);

    let status = gl.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .ok_or(CmcError::missing_val("Link status"))?;

    if !status {
        let log = gl.get_program_info_log(&program).ok_or(CmcError::missing_val("Program log"))?;
        Err(CmcError::ShaderLink{ log })?;
    }
    Ok(program)
}

pub struct CmcVertex(Vertex);

impl From<&Vertex> for CmcVertex {
    fn from(vertex: &Vertex) -> CmcVertex {
        CmcVertex(vertex.clone())
    }
}

impl From<CmcVertex> for Vec<f32>{
    fn from(vertex: CmcVertex) -> Vec<f32> {
        vec![vertex.0.x as f32, vertex.0.y as f32, vertex.0.z as f32]
    }
}

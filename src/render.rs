use crate::error::{CmcError, CmcResult};
use js_sys::WebAssembly;
use nalgebra::{Matrix4, Vector3};
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as WebGL;
use web_sys::*;

pub struct Renderer {
    program: WebGlProgram,
    rect_vertice_array_length: usize,
    rect_vertice_buffer: WebGlBuffer,
    u_color: WebGlUniformLocation,
    u_opacity: WebGlUniformLocation,
    u_transform: WebGlUniformLocation,
}

impl Renderer {
    pub fn new(gl: &WebGlRenderingContext) -> CmcResult<Self> {
        let program = gl.create_program().ok_or(CmcError::missing_val("create program"))?;
        let vert_shader = compile_shader(&gl, WebGL::VERTEX_SHADER, crate::shaders::vertex::SHADER)?;
        let frag_shader = compile_shader(&gl, WebGL::FRAGMENT_SHADER, crate::shaders::fragment::SHADER)?;

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
        let vertices_rect: [f32; 6] = [
            0.25, 0.75,
            0.25, 0.25,
            0.75, 0.75,
        ];

        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let vertices_location = vertices_rect.as_ptr() as u32 / 4;
        let vert_array = js_sys::Float32Array::new(&memory_buffer).subarray(
            vertices_location,
            vertices_location + vertices_rect.len() as u32);
        let rect_vertice_buffer = gl.create_buffer().ok_or(CmcError::missing_val("Failed to create buffer"))?;
        gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&rect_vertice_buffer));
        gl.buffer_data_with_array_buffer_view(WebGL::ARRAY_BUFFER, &vert_array, WebGL::STATIC_DRAW);
        let u_color = gl.get_uniform_location(&program, "uColor")
            .ok_or(CmcError::missing_val("uColor"))?;
        let u_opacity = gl.get_uniform_location(&program, "uOpacity")
            .ok_or(CmcError::missing_val("uOpacity"))?;
        let u_transform = gl.get_uniform_location(&program, "uTransform")
            .ok_or(CmcError::missing_val("uTransform"))?;
        Ok(Renderer {
            program,
            rect_vertice_array_length: vertices_rect.len(),
            rect_vertice_buffer,
            u_color,
            u_opacity,
            u_transform,
        })
    }

    pub fn render(
        &self,
        gl: &WebGlRenderingContext,
        bottom: f32,
        top: f32,
        left: f32,
        right: f32,
        canvas_height: f32,
        canvas_width: f32,
    ) {
        gl.use_program(Some(&self.program));

        gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&self.rect_vertice_buffer));
        gl.vertex_attrib_pointer_with_i32(0, 2, WebGL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        gl.uniform4f(
            Some(&self.u_color),
            0., //r
            0.5,//g
            0.5,//b
            1.0,//a
        );

        gl.uniform1f(Some(&self.u_opacity), 1.);

        let transform_mat = Matrix4::new_nonuniform_scaling(&Vector3::new(
            2. * (right - left) / canvas_width,
            2. * (top - bottom) / canvas_height,
            0.))
            .append_translation(&Vector3::new(
            2. * left / canvas_width - 1.,
            2. * bottom / canvas_height - 1.,
            0.,
            ));
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_transform), false, transform_mat.as_slice());

        gl.draw_arrays(WebGL::TRIANGLES, 0, (self.rect_vertice_array_length / 2) as i32);

    }
}

fn compile_shader(
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


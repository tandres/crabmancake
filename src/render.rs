use crate::{common::compile_shader, error::{CmcError, CmcResult}};
use js_sys::WebAssembly;
use nalgebra::{Isometry3, Perspective3, Point3, Vector3};
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as WebGL;
use web_sys::*;

pub struct SimpleRenderer {
    program: WebGlProgram,
    rect_vertice_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,
    index_count: i32,
    u_color: WebGlUniformLocation,
    u_opacity: WebGlUniformLocation,
    u_transform: WebGlUniformLocation,
}

impl SimpleRenderer {
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
        let vertices_rect: [f32; 12] = [
            -0.5, -0.5, 0.,
            -0.5, 0.5, 0.,
            0.5, -0.5, 0.,
            0.5, 0.5, 0.,
        ];

        let indices_rect: [u16; 6] = [0, 1, 2, 2, 1, 3];

        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let vertices_location = vertices_rect.as_ptr() as u32 / 4;
        let vert_array = js_sys::Float32Array::new(&memory_buffer).subarray(
            vertices_location,
            vertices_location + vertices_rect.len() as u32);
        let rect_vertice_buffer = gl.create_buffer().ok_or(CmcError::missing_val("Failed to create buffer"))?;
        gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&rect_vertice_buffer));
        gl.buffer_data_with_array_buffer_view(WebGL::ARRAY_BUFFER, &vert_array, WebGL::STATIC_DRAW);

        let indices_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let indices_location = indices_rect.as_ptr() as u32 / 2;
        let indices_array = js_sys::Uint16Array::new(&indices_buffer).subarray(
            indices_location,
            indices_location + indices_rect.len() as u32);
        let indices_buffer = gl.create_buffer().ok_or(CmcError::missing_val("Failed to create buffer"))?;
        gl.bind_buffer(WebGL::ELEMENT_ARRAY_BUFFER, Some(&indices_buffer));
        gl.buffer_data_with_array_buffer_view(WebGL::ELEMENT_ARRAY_BUFFER, &indices_array, WebGL::STATIC_DRAW);
        let u_color = gl.get_uniform_location(&program, "uColor")
            .ok_or(CmcError::missing_val("uColor"))?;
        let u_opacity = gl.get_uniform_location(&program, "uOpacity")
            .ok_or(CmcError::missing_val("uOpacity"))?;
        let u_transform = gl.get_uniform_location(&program, "uTransform")
            .ok_or(CmcError::missing_val("uTransform"))?;
        Ok(SimpleRenderer {
            program,
            rect_vertice_buffer,
            index_buffer: indices_buffer,
            index_count: indices_array.length() as i32,
            u_color,
            u_opacity,
            u_transform,
        })
    }
}

pub trait Renderer {
    fn render(&self, gl: &WebGlRenderingContext, canvas_height: f32, canvas_width: f32, location: &Vector3<f32>, rotation: &Vector3<f32>);
}

impl Renderer for SimpleRenderer {
    fn render(
        &self,
        gl: &WebGlRenderingContext,
        canvas_height: f32,
        canvas_width: f32,
        location: &Vector3<f32>,
        rotation: &Vector3<f32>,
    ) {
        gl.use_program(Some(&self.program));

        gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&self.rect_vertice_buffer));
        gl.vertex_attrib_pointer_with_i32(0, 3, WebGL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        gl.uniform4f(
            Some(&self.u_color),
            0., //r
            0.5,//g
            0.5,//b
            1.0,//a
        );
        gl.uniform1f(Some(&self.u_opacity), 1.);
        let aspect: f32 = canvas_width / canvas_height;
        pub const FIELD_OF_VIEW: f32 = 45. * std::f32::consts::PI / 180.; //in radians
        pub const Z_FAR: f32 = 1000.;
        pub const Z_NEAR: f32 = 1.0;
        let eye    = Point3::new(0.0, 0.0, 3.0);
        let target = Point3::new(0.0, 0.0, 1.0);
        let view   = Isometry3::look_at_rh(&eye, &target, &Vector3::y());

        let model      = Isometry3::new(location.clone(), rotation.clone());
        let projection = Perspective3::new(aspect, FIELD_OF_VIEW, Z_NEAR, Z_FAR);
        let mvp = projection.as_matrix() * (view * model).to_homogeneous();

        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_transform), false, mvp.as_slice());

        gl.bind_buffer(WebGL::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));

        gl.draw_elements_with_i32(WebGL::TRIANGLES, self.index_count, WebGL::UNSIGNED_SHORT, 0);

    }
}



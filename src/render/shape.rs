use crate::error::{CmcError, CmcResult};
use super::{Renderer, common::build_program};
use js_sys::WebAssembly;
use nalgebra::{Isometry3, Perspective3, Vector3};
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as WebGL;
use web_sys::*;

const VERT_SHADER: &str = r#"
    attribute vec4 aPosition;
    attribute vec3 aNormal;

    uniform mat4 uTransform;
    uniform mat4 uNormalRot;
    uniform vec3 uColor;
    uniform vec3 uAmbient;
    uniform vec3 uDirectional;
    uniform vec3 uDirectionalVector;
    varying lowp vec4 vColor;

    void main() {
        gl_Position = uTransform * aPosition;

        vec3 directionalVector = normalize(uDirectionalVector);
        vec4 transformedNormal = uNormalRot * vec4(aNormal, 1.0);

        float directional = max(dot(transformedNormal.xyz, directionalVector), 0.0);
        vec3 vLighting = uAmbient + (uDirectional * directional);

        vColor = vec4(uColor * vLighting, 1.0);
    }
"#;

const FRAG_SHADER: &str = r#"
    precision mediump float;

    uniform float uOpacity;

    varying lowp vec4 vColor;

    void main() {
        gl_FragColor = vec4(vColor.r, vColor.g, vColor.b, vColor.a * uOpacity);
    }
"#;

pub struct ShapeRenderer {
    pub name: String,
    program: WebGlProgram,
    vertice_buffer: WebGlBuffer,
    normals_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,
    index_count: i32,
    u_color: WebGlUniformLocation,
    u_opacity: WebGlUniformLocation,
    u_transform: WebGlUniformLocation,
    u_ambient: WebGlUniformLocation,
    u_directional: WebGlUniformLocation,
    u_directional_vector: WebGlUniformLocation,
    u_normal_rot: WebGlUniformLocation,
}

impl ShapeRenderer {
    pub fn new(name: &String, gl: &WebGlRenderingContext, vertices: Vec<f32>, indices: Vec<u16>, normals: Vec<f32>) -> CmcResult<Self> {
        let program = build_program(gl, VERT_SHADER, FRAG_SHADER)?;
        let vertices_rect = vertices.as_slice();

        let indices_rect = indices.as_slice();

        let normals_rect = normals.as_slice();

        let vertices_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let vertices_location = vertices_rect.as_ptr() as u32 / 4;
        let vert_array = js_sys::Float32Array::new(&vertices_buffer).subarray(
            vertices_location,
            vertices_location + vertices_rect.len() as u32);
        let vertice_buffer = gl.create_buffer().ok_or(CmcError::missing_val("Failed to create buffer"))?;
        gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&vertice_buffer));
        gl.buffer_data_with_array_buffer_view(WebGL::ARRAY_BUFFER, &vert_array, WebGL::STATIC_DRAW);

        let normals_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let normals_location = normals_rect.as_ptr() as u32 / 4;
        let normals_array = js_sys::Float32Array::new(&normals_buffer).subarray(
            normals_location,
            normals_location + normals_rect.len() as u32);
        let normals_buffer = gl.create_buffer().ok_or(CmcError::missing_val("Failed to create normals buffer"))?;
        gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&normals_buffer));
        gl.buffer_data_with_array_buffer_view(WebGL::ARRAY_BUFFER, &normals_array, WebGL::STATIC_DRAW);

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
        let u_normal_rot = gl.get_uniform_location(&program, "uNormalRot")
            .ok_or(CmcError::missing_val("uNormalRot"))?;
        let u_ambient = gl.get_uniform_location(&program, "uAmbient")
            .ok_or(CmcError::missing_val("uAmbient"))?;
        let u_directional = gl.get_uniform_location(&program, "uDirectional")
            .ok_or(CmcError::missing_val("uDirectional"))?;
        let u_directional_vector = gl.get_uniform_location(&program, "uDirectionalVector")
            .ok_or(CmcError::missing_val("uDirectionalVector"))?;
        Ok(ShapeRenderer {
            name: name.clone(),
            program,
            vertice_buffer,
            index_buffer: indices_buffer,
            index_count: indices_array.length() as i32,
            normals_buffer,
            u_color,
            u_opacity,
            u_transform,
            u_ambient,
            u_directional,
            u_directional_vector,
            u_normal_rot,
        })
    }
}

impl Renderer for ShapeRenderer {
    fn render(
        &self,
        gl: &WebGlRenderingContext,
        view: &Isometry3<f32>,
        projection: &Perspective3<f32>,
        location: &Vector3<f32>,
        rotation: &Vector3<f32>,
    ) {
        gl.use_program(Some(&self.program));

        gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&self.vertice_buffer));
        gl.vertex_attrib_pointer_with_i32(0, 3, WebGL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&self.normals_buffer));
        gl.vertex_attrib_pointer_with_i32(1, 3, WebGL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(1);

        gl.uniform3f(
            Some(&self.u_color),
            0., //r
            0.5,//g
            0.5,//b
        );
        gl.uniform1f(Some(&self.u_opacity), 1.);
        let model = Isometry3::new(location.clone(), rotation.clone());
        let normal_rot = Isometry3::rotation(rotation.clone()).inverse();
        let mvp = projection.as_matrix() * (view * model).to_homogeneous();

        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_transform), false, mvp.as_slice());
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_normal_rot), false, normal_rot.to_homogeneous().as_slice());

        let ambient_light = vec![0.1, 0.1, 0.1];
        gl.uniform3fv_with_f32_array(Some(&self.u_ambient), ambient_light.as_slice());
        let directional_light = vec![1., 1., 1.];
        gl.uniform3fv_with_f32_array(Some(&self.u_directional), directional_light.as_slice());
        let directional_vector = vec![0.5, 0.5, 0.5];
        gl.uniform3fv_with_f32_array(Some(&self.u_directional_vector), directional_vector.as_slice());

        gl.bind_buffer(WebGL::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));

        gl.draw_elements_with_i32(WebGL::TRIANGLES, self.index_count, WebGL::UNSIGNED_SHORT, 0);
    }
}



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

    uniform mat4 uView;
    uniform mat4 uProjection;
    uniform mat4 uModel;
    varying vec3 vNormal;

    void main() {
        gl_Position = uProjection * ((uView * uModel) * aPosition);

        vNormal = mat3(uModel) * aNormal;
    }
"#;

const FRAG_SHADER: &str = r#"
    precision mediump float;
    varying vec3 vNormal;

    uniform vec3 uAmbientLight;
    uniform vec3 uDirLightColor;
    uniform vec4 uColor;
    uniform vec3 uDirLightVector;

    void main() {
        vec3 normal = normalize(vNormal);

        vec3 dirLightVector = normalize(uDirLightVector);

        float directional = max(dot(normal, dirLightVector), 0.0);
        vec3 lighting = uAmbientLight + (directional * uDirLightColor);

        gl_FragColor = uColor * vec4(lighting, 1.0);
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
    u_model: WebGlUniformLocation,
    u_view: WebGlUniformLocation,
    u_projection: WebGlUniformLocation,
    u_ambient_light: WebGlUniformLocation,
    u_dir_light_color: WebGlUniformLocation,
    u_dir_light_vector: WebGlUniformLocation,
    normal_renderer: Option<NormalRenderer>,
}

impl ShapeRenderer {
    pub fn new(name: &String, gl: &WebGlRenderingContext, vertices: Vec<f32>, indices: Vec<u16>, normals: Vec<f32>) -> CmcResult<Self> {
        // let normal_renderer = NormalRenderer::new(&"Normal".to_string(), gl, &vertices, &indices, &normals)?;
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
        log::info!("Normals: {}", normals_rect.len());
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
        let u_model = gl.get_uniform_location(&program, "uModel")
            .ok_or(CmcError::missing_val("uModel"))?;

        let u_view = gl.get_uniform_location(&program, "uView")
            .ok_or(CmcError::missing_val("uView"))?;
        let u_projection = gl.get_uniform_location(&program, "uProjection")
            .ok_or(CmcError::missing_val("uProjection"))?;

        let u_ambient_light = gl.get_uniform_location(&program, "uAmbientLight")
            .ok_or(CmcError::missing_val("uAmbientLight"))?;
        let u_dir_light_color = gl.get_uniform_location(&program, "uDirLightColor")
            .ok_or(CmcError::missing_val("uDirLightColor"))?;
        let u_dir_light_vector = gl.get_uniform_location(&program, "uDirLightVector")
            .ok_or(CmcError::missing_val("uDirLightVector"))?;
        Ok(ShapeRenderer {
            name: name.clone(),
            program,
            vertice_buffer,
            index_buffer: indices_buffer,
            index_count: indices_array.length() as i32,
            normals_buffer,
            u_color,
            u_model,
            u_view,
            u_projection,
            u_ambient_light,
            u_dir_light_color,
            u_dir_light_vector,
            normal_renderer: None,
            // normal_renderer: Some(normal_renderer),
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

        gl.uniform4f(
            Some(&self.u_color),
            0., //r
            0.5,//g
            0.5,//b
            1.,//a
        );
        let model_mat = Isometry3::new(location.clone(), rotation.clone()).to_homogeneous();
        let projection_mat = projection.to_homogeneous();
        //let projection_mat = projection.as_matrix();
        let view_mat = view.to_homogeneous();
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_model), false, model_mat.as_slice());
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_view), false, view_mat.as_slice());
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_projection), false, projection_mat.as_slice());

        let ambient_light = vec![0.1, 0.1, 0.1];
        gl.uniform3fv_with_f32_array(Some(&self.u_ambient_light), ambient_light.as_slice());
        let directional_light = vec![1., 1., 1.];
        gl.uniform3fv_with_f32_array(Some(&self.u_dir_light_color), directional_light.as_slice());
        let directional_light_vector = -Vector3::new(0., 0., 1.);
        gl.uniform3fv_with_f32_array(Some(&self.u_dir_light_vector), directional_light_vector.as_slice());

        gl.bind_buffer(WebGL::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));

        gl.draw_elements_with_i32(WebGL::TRIANGLES, self.index_count, WebGL::UNSIGNED_SHORT, 0);

        if let Some(normal_renderer) = &self.normal_renderer {
            normal_renderer.render(gl, view, projection, location, rotation);
        }
    }
}

const NORMAL_VERT_SHADER: &str = r#"
    attribute vec4 aPosition;
    attribute vec3 aNormal;

    uniform mat4 uView;
    uniform mat4 uProjection;
    uniform mat4 uModel;

    void main() {
        vec4 position = aPosition;
        position.xyz += aNormal;
        gl_Position = uProjection * ((uView * uModel) * position);
    }
"#;

const NORMAL_FRAG_SHADER: &str = r#"
    precision mediump float;
    uniform vec4 uColor;

    void main() {
        gl_FragColor = uColor;
    }
"#;

pub struct NormalRenderer {
    pub name: String,
    program: WebGlProgram,
    vertex_buffer: WebGlBuffer,
    vertex_count: i32,
    u_color: WebGlUniformLocation,
    u_view: WebGlUniformLocation,
    u_projection: WebGlUniformLocation,
    u_model: WebGlUniformLocation,
    normals_buffer: WebGlBuffer,
}

impl NormalRenderer {
    pub fn new(name: &String, gl: &WebGlRenderingContext, vertices: &Vec<f32>, indices: &Vec<u16>, normals: &Vec<f32>) -> CmcResult<Self> {
        let program = build_program(gl, NORMAL_VERT_SHADER, NORMAL_FRAG_SHADER)?;

        let vertices_rect = vertices.as_slice();
        let normals_rect = normals.as_slice();
        let mut normals_mut : Vec<f32> = Vec::new();
        let mut normal_lines : Vec<f32> = Vec::new();
        log::trace!("Indices len: {}", indices.len());
        for i in 0..indices.len() {
            let index = (3 * indices[i]) as usize;
            log::trace!("I: {} indices[i]: {} Index: {}", i, indices[i], index);
            let first = Vector3::new(vertices_rect[index + 0], vertices_rect[index + 1], vertices_rect[index + 2]);
            let norm = Vector3::new(normals_rect[i * 3], normals_rect[i * 3 + 1], normals_rect[i * 3 + 2]);
            normal_lines.extend(first.as_slice());
            normals_mut.extend(Vector3::zeros().as_slice());
            normal_lines.extend(first.as_slice());
            normals_mut.extend(norm.as_slice());
        }
        let normal_lines = normal_lines.as_slice();

        let normals_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let normals_location = normals_mut.as_ptr() as u32 / 4;
        let normals_array = js_sys::Float32Array::new(&normals_buffer).subarray(
            normals_location,
            normals_location + normals_mut.len() as u32);
        let normals_buffer = gl.create_buffer().ok_or(CmcError::missing_val("Failed to create normals buffer"))?;
        gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&normals_buffer));
        gl.buffer_data_with_array_buffer_view(WebGL::ARRAY_BUFFER, &normals_array, WebGL::STATIC_DRAW);

        let vertex_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let vertex_location = normal_lines.as_ptr() as u32 / 4;
        let vertex_array = js_sys::Float32Array::new(&vertex_buffer).subarray(
            vertex_location,
            vertex_location + normal_lines.len() as u32);
        let vertex_buffer = gl.create_buffer().ok_or(CmcError::missing_val("Failed to create normal lines buffer"))?;
        gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.buffer_data_with_array_buffer_view(WebGL::ARRAY_BUFFER, &vertex_array, WebGL::STATIC_DRAW);

        let u_color = gl.get_uniform_location(&program, "uColor")
            .ok_or(CmcError::missing_val("uColor"))?;
        let u_view = gl.get_uniform_location(&program, "uView")
            .ok_or(CmcError::missing_val("uView"))?;
        let u_projection = gl.get_uniform_location(&program, "uProjection")
            .ok_or(CmcError::missing_val("uProjection"))?;
        let u_model = gl.get_uniform_location(&program, "uModel")
            .ok_or(CmcError::missing_val("uModel"))?;
        let vertex_count = vertex_array.length() as i32;
        log::info!("{} normals with {} vertices", indices.len(), vertex_count);
        Ok(NormalRenderer {
            name: name.clone(),
            program,
            u_color,
            u_model,
            u_projection,
            u_view,
            vertex_buffer,
            vertex_count,
            normals_buffer,
        })
    }
}

impl Renderer for NormalRenderer {
    fn render(
        &self,
        gl: &WebGlRenderingContext,
        view: &Isometry3<f32>,
        projection: &Perspective3<f32>,
        location: &Vector3<f32>,
        rotation: &Vector3<f32>,
    ) {
        gl.use_program(Some(&self.program));

        gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&self.vertex_buffer));
        gl.vertex_attrib_pointer_with_i32(0, 3, WebGL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&self.normals_buffer));
        gl.vertex_attrib_pointer_with_i32(1, 3, WebGL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(1);

        gl.uniform4f(Some(&self.u_color), 1., 1., 1., 1.);

        let model = Isometry3::new(location.clone(), rotation.clone()).to_homogeneous();
        let projection = projection.as_matrix();
        let view = view.to_homogeneous();
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_view), false, view.as_slice());
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_projection), false, projection.as_slice());
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_model), false, model.as_slice());

        gl.draw_arrays(WebGL::LINES, 0, self.vertex_count / 3 );
    }
}

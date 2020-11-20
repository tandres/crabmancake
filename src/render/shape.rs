use crate::error::{CmcError, CmcResult};
use super::{Light, Renderer, common::build_program};
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
    varying vec3 vFragLoc;

    void main() {
        gl_Position = uProjection * ((uView * uModel) * aPosition);
        vFragLoc = vec3(uModel * aPosition);
        vNormal = mat3(uModel) * aNormal;
    }
"#;
const MAX_POINT_LIGHTS: usize = 10;
const FRAG_SHADER: &str = r#"
    #define MAX_POINT_LIGHTS 10
    precision mediump float;
    varying vec3 vNormal;
    varying vec3 vFragLoc;

    uniform vec3 uAmbientLight;
    uniform vec4 uColor;

    struct PointLight {
        vec3 color;
        vec3 location;
    };
    uniform PointLight point_lights[MAX_POINT_LIGHTS];

    void main() {
        vec3 normal = normalize(vNormal);

        vec3 lighting = uAmbientLight;
        for(int i = 0; i < MAX_POINT_LIGHTS; i++) {
            vec3 dirLightVector = normalize(point_lights[i].location - vFragLoc);
            float directional = max(dot(normal, dirLightVector), 0.0);
            lighting += directional * point_lights[i].color;
        }

        gl_FragColor = uColor * vec4(lighting, 1.0);
    }
"#;

pub struct PointLight {
    color: WebGlUniformLocation,
    location: WebGlUniformLocation,
}

impl PointLight {
    fn new_at_index(gl: &WebGlRenderingContext, program: &WebGlProgram, array_name: &str, index: usize) -> CmcResult<Self> {
        let color_name = format!("{}[{}].color", array_name, index);
        let location_name = format!("{}[{}].location", array_name, index);
        let color = gl.get_uniform_location(program, color_name.as_str())
            .ok_or(CmcError::missing_val(color_name))?;
        let location = gl.get_uniform_location(program, location_name.as_str())
            .ok_or(CmcError::missing_val(location_name))?;
        Ok(Self { color, location })
    }
}

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
    point_lights: Vec<PointLight>,
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
        log::info!("U_ambient_light: {:?}", u_ambient_light);
        let mut point_lights: Vec<PointLight> = Vec::new();
        for i in 0..MAX_POINT_LIGHTS {
            point_lights.push(PointLight::new_at_index(gl, &program, "point_lights", i)?);
        }
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
            point_lights,
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
        lights: &Vec<Light>,
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
            1., //r
            1.,//g
            1.,//b
            1.,//a
        );
        let model_mat = Isometry3::new(location.clone(), rotation.clone()).to_homogeneous();
        let projection_mat = projection.to_homogeneous();
        //let projection_mat = projection.as_matrix();
        let view_mat = view.to_homogeneous();
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_model), false, model_mat.as_slice());
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_view), false, view_mat.as_slice());
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_projection), false, projection_mat.as_slice());

        let ambient_light = vec![0., 0., 0.1];
        gl.uniform3fv_with_f32_array(Some(&self.u_ambient_light), ambient_light.as_slice());
        for (index, light) in lights.iter().enumerate() {
            match light {
                Light::Point{ color, location } => {
                    let color_location = &self.point_lights[index].color;
                    let location_location = &self.point_lights[index].location;
                    gl.uniform3fv_with_f32_array(Some(color_location), color.as_slice());
                    gl.uniform3fv_with_f32_array(Some(location_location), location.as_slice());
                },
            }
        }
        gl.bind_buffer(WebGL::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));

        gl.draw_elements_with_i32(WebGL::TRIANGLES, self.index_count, WebGL::UNSIGNED_SHORT, 0);
    }
}


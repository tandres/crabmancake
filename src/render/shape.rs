use crate::error::{CmcError, CmcResult};
use super::{Light, common::build_program, gob::{Gob, GobDataAttribute}};
use js_sys::WebAssembly;
use nalgebra::{Isometry3, Perspective3, Vector3};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as WebGL;
use web_sys::*;

const VERT_SHADER: &str = r#"
    attribute vec4 aPosition;
    attribute vec3 aNormal;
    attribute vec2 aTextureCoord;

    uniform mat4 uView;
    uniform mat4 uProjection;
    uniform mat4 uModel;
    varying vec3 vNormal;
    varying vec3 vFragLoc;
    varying vec2 vTextureCoord;

    void main() {
        gl_Position = uProjection * ((uView * uModel) * aPosition);
        vFragLoc = vec3(uModel * aPosition);
        vNormal = mat3(uModel) * aNormal;
        vTextureCoord = aTextureCoord;
    }
"#;
const MAX_LIGHTS: usize = 10;
const FRAG_SHADER: &str = r#"
    #define MAX_LIGHTS 10

    precision mediump float;
    varying vec3 vNormal;
    varying vec3 vFragLoc;
    varying vec2 vTextureCoord;

    uniform vec3 uAmbientLight;
    uniform vec3 uEyeLocation;
    uniform sampler2D uTexture;

    struct Light {
        vec3 color;
        vec3 location;
        vec3 direction;
        float inner_limit;
        float outer_limit;

        float intensity;

        vec3 attenuator;
    };
    uniform Light spot_lights[MAX_LIGHTS];

    void main() {
        vec3 normal = normalize(vNormal);
        vec3 fragment_to_view = normalize(uEyeLocation - vFragLoc);

        vec3 lighting = uAmbientLight;

        for(int j = 0; j < MAX_LIGHTS; j++) {
            vec3 light_location = spot_lights[j].location;
            vec3 light_direction = spot_lights[j].direction;
            vec3 light_color = spot_lights[j].color;
            float outer_limit = spot_lights[j].outer_limit;
            float inner_limit = spot_lights[j].inner_limit;
            vec3 attenuator = spot_lights[j].attenuator;
            float intensity = spot_lights[j].intensity;

            vec3 fragment_to_light = normalize(light_location - vFragLoc);
            float dot_f2l_ldir = dot(fragment_to_light, normalize(-light_direction));
            float inLight = smoothstep(outer_limit, inner_limit, dot_f2l_ldir);
            float diffuse_directional = inLight * max(dot(normal, fragment_to_light), 0.0);
            float specular = 0.0;
            if (diffuse_directional > 0.0) {
                vec3 half_vector = normalize(fragment_to_light + fragment_to_view);
                float viewable_reflection = dot(normal, half_vector);
                specular = pow(max(viewable_reflection, 0.0), 32.0);
            }
            float distance    = length(light_location - vFragLoc);
            float attenuation = max(1.0, intensity) / (1.0 + attenuator.y * distance +
    		    attenuator.z * (distance * distance));
            lighting += (diffuse_directional + specular) * spot_lights[j].color * attenuation;
        }

        gl_FragColor = texture2D(uTexture, vTextureCoord) * vec4(lighting, 1.0);
    }
"#;

struct RenderLight {
    color: WebGlUniformLocation,
    location: WebGlUniformLocation,
    direction: WebGlUniformLocation,
    inner_limit: WebGlUniformLocation,
    outer_limit: WebGlUniformLocation,
    intensity: WebGlUniformLocation,
    attenuator: WebGlUniformLocation,
}

impl RenderLight {
    fn new_at_index(gl: &WebGlRenderingContext, program: &WebGlProgram, array_name: &str, index: usize) -> CmcResult<Self> {
        let color_name = format!("{}[{}].color", array_name, index);
        let location_name = format!("{}[{}].location", array_name, index);
        let direction_name = format!("{}[{}].direction", array_name, index);
        let inner_limit_name = format!("{}[{}].inner_limit", array_name, index);
        let outer_limit_name = format!("{}[{}].outer_limit", array_name, index);
        let intensity_name = format!("{}[{}].intensity", array_name, index);
        let attenuator_name = format!("{}[{}].attenuator", array_name, index);
        let color = gl.get_uniform_location(program, color_name.as_str())
            .ok_or(CmcError::missing_val(color_name))?;
        let location = gl.get_uniform_location(program, location_name.as_str())
            .ok_or(CmcError::missing_val(location_name))?;
        let direction = gl.get_uniform_location(program, direction_name.as_str())
            .ok_or(CmcError::missing_val(direction_name))?;
        let inner_limit = gl.get_uniform_location(program, inner_limit_name.as_str())
            .ok_or(CmcError::missing_val(inner_limit_name))?;
        let outer_limit = gl.get_uniform_location(program, outer_limit_name.as_str())
            .ok_or(CmcError::missing_val(outer_limit_name))?;
        let intensity = gl.get_uniform_location(program, intensity_name.as_str())
            .ok_or(CmcError::missing_val(intensity_name))?;
        let attenuator = gl.get_uniform_location(program, attenuator_name.as_str())
            .ok_or(CmcError::missing_val(attenuator_name))?;
        Ok(Self { color, location, inner_limit, outer_limit, direction, intensity, attenuator})
    }

    fn populate_with(&self, gl: &WebGlRenderingContext, source_light: &Light) {
        match source_light {
            Light::Spot{ color, location, direction, inner_limit, outer_limit, intensity, attenuator } => {
                let color_location = &self.color;
                let location_location = &self.location;
                let direction_location = &self.direction;
                let inner_limit_location = &self.inner_limit;
                let outer_limit_location = &self.outer_limit;
                let intensity_location = &self.intensity;
                let attenuator_location = &self.attenuator;
                gl.uniform3fv_with_f32_array(Some(color_location), color.as_slice());
                gl.uniform3fv_with_f32_array(Some(location_location), location.as_slice());
                gl.uniform3fv_with_f32_array(Some(direction_location), direction.as_slice());
                gl.uniform1f(Some(inner_limit_location), *inner_limit);
                gl.uniform1f(Some(outer_limit_location), *outer_limit);
                gl.uniform1f(Some(intensity_location), *intensity);
                gl.uniform3fv_with_f32_array(Some(attenuator_location), &attenuator[..]);
            },
        }
    }
}

pub struct ShapeRenderer {
    pub name: String,
    program: WebGlProgram,
    gob: Gob,
    geometry_buffers: HashMap<usize, WebGlBuffer>,
    u_texture: WebGlUniformLocation,
    u_model: WebGlUniformLocation,
    u_view: WebGlUniformLocation,
    u_projection: WebGlUniformLocation,
    u_ambient_light: WebGlUniformLocation,
    u_eye: WebGlUniformLocation,
    lights: Vec<RenderLight>,
    texture: WebGlTexture,
}

fn attr_location(attr_data: &GobDataAttribute) -> Option<u32> {
    match attr_data {
        GobDataAttribute::Positions => Some(0),
        GobDataAttribute::TexCoords(0) => Some(2),
        GobDataAttribute::Normals => Some(1),
        _ => None,
    }
}

impl ShapeRenderer {
    pub fn new(name: &String, gl: &WebGlRenderingContext, mut gob: Gob, texture_image: Vec<u8>, image_width: u32, image_height: u32) -> CmcResult<Self> {
        let program = build_program(gl, VERT_SHADER, FRAG_SHADER)?;
        let mut geometry_buffers = HashMap::new();
        let js_memory = wasm_bindgen::memory().dyn_into::<WebAssembly::Memory>()?.buffer();
        let js_memory_2 = js_sys::Uint8Array::new(&js_memory);
        for (index, gob_buffer) in gob.buffers.iter() {
            log::debug!("Index: {}", index);
            let gb_slice = gob_buffer.data.as_slice();
            let gb_location = gb_slice.as_ptr() as u32;
            let gb_len = (gb_slice.len() * std::mem::size_of::<u8>()) as u32;
            let js_buf = js_memory_2.subarray(gb_location, gb_location + gb_len);
            log::debug!("JS_BUF: {:X?}", js_buf.to_vec().as_slice());
            let gl_buf = gl.create_buffer()
                .ok_or(CmcError::missing_val(format!("Failed to create buffer index: {}", index)))?;
            gl.bind_buffer(gob_buffer.target.to_gl(), Some(&gl_buf));
            gl.buffer_data_with_array_buffer_view(gob_buffer.target.to_gl(), &js_buf, WebGL::STATIC_DRAW);
            geometry_buffers.insert(*index, gl_buf);
        }
        log::debug!("Buffers: {}", geometry_buffers.len());

        for (attr, gob_data_access) in gob.accessors.iter_mut() {
            gob_data_access.gl_attribute_index = attr_location(&attr);
        }

        let u_texture = gl.get_uniform_location(&program, "uTexture")
            .ok_or(CmcError::missing_val("uTexture"))?;
        let texture = gl.create_texture()
            .ok_or(CmcError::missing_val("Texture creation"))?;
        let texture_buf = texture_image;
        gl.bind_texture(WebGL::TEXTURE_2D, Some(&texture));
        gl.tex_parameteri(WebGL::TEXTURE_2D, WebGL::TEXTURE_WRAP_S, WebGL::MIRRORED_REPEAT as i32);
        gl.tex_parameteri(WebGL::TEXTURE_2D, WebGL::TEXTURE_WRAP_T, WebGL::MIRRORED_REPEAT as i32);
        let image_width = image_width as i32;
        let image_height = image_height as i32;
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(WebGL::TEXTURE_2D, 0, WebGL::RGBA as i32, image_width, image_height, 0, WebGL::RGBA, WebGL::UNSIGNED_BYTE, Some(texture_buf.as_slice()))?;
        gl.generate_mipmap(WebGL::TEXTURE_2D);
        let u_model = gl.get_uniform_location(&program, "uModel")
            .ok_or(CmcError::missing_val("uModel"))?;

        let u_view = gl.get_uniform_location(&program, "uView")
            .ok_or(CmcError::missing_val("uView"))?;
        let u_projection = gl.get_uniform_location(&program, "uProjection")
            .ok_or(CmcError::missing_val("uProjection"))?;

        let u_eye = gl.get_uniform_location(&program, "uEyeLocation")
            .ok_or(CmcError::missing_val("uEyeLocation"))?;
        let u_ambient_light = gl.get_uniform_location(&program, "uAmbientLight")
            .ok_or(CmcError::missing_val("uAmbientLight"))?;
        let mut lights: Vec<RenderLight> = Vec::new();
        for i in 0..MAX_LIGHTS {
            lights.push(RenderLight::new_at_index(gl, &program, "spot_lights", i)?);
        }
        Ok(ShapeRenderer {
            name: name.clone(),
            gob,
            program,
            geometry_buffers,
            u_texture,
            u_model,
            u_view,
            u_projection,
            u_ambient_light,
            lights,
            u_eye,
            texture,
        })
    }

    pub fn render(
        &self,
        gl: &WebGlRenderingContext,
        view: &Isometry3<f32>,
        eye: &Vector3<f32>,
        projection: &Perspective3<f32>,
        location: &Vector3<f32>,
        rotation: &Vector3<f32>,
        lights: &Vec<Light>,
    ) {
        gl.use_program(Some(&self.program));
        for (_key, gob_acc) in self.gob.accessors.iter().filter(|v| *v.0 != GobDataAttribute::Indices) {
            if let Some(gl_attr_index) = gob_acc.gl_attribute_index {
                gl.bind_buffer(WebGL::ARRAY_BUFFER, Some(&self.geometry_buffers[&gob_acc.buffer_index]));
                gl.vertex_attrib_pointer_with_i32(gl_attr_index, gob_acc.num_items, gob_acc.data_type, gob_acc.normalized, gob_acc.stride, gob_acc.offset);
                gl.enable_vertex_attrib_array(gl_attr_index);
            }
        }

        gl.active_texture(WebGL::TEXTURE0);
        gl.bind_texture(WebGL::TEXTURE_2D, Some(&self.texture));
        gl.uniform1i(Some(&self.u_texture), 0);

        let model_mat = Isometry3::new(location.clone(), rotation.clone()).to_homogeneous();
        let projection_mat = projection.to_homogeneous();
        let view_mat = view.to_homogeneous();
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_model), false, model_mat.as_slice());
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_view), false, view_mat.as_slice());
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_projection), false, projection_mat.as_slice());
        gl.uniform3fv_with_f32_array(Some(&self.u_eye), eye.as_slice());

        let ambient_light = vec![0.1, 0.1, 0.1];
        gl.uniform3fv_with_f32_array(Some(&self.u_ambient_light), ambient_light.as_slice());
        for (index, light) in lights.iter().enumerate() {
            self.lights[index].populate_with(gl, light);
        }

        let gob_acc = self.gob.accessors.get(&GobDataAttribute::Indices).unwrap();
        gl.bind_buffer(WebGL::ELEMENT_ARRAY_BUFFER, Some(&self.geometry_buffers[&gob_acc.buffer_index]));

        gl.draw_elements_with_i32(WebGL::TRIANGLES, gob_acc.count as i32, gob_acc.data_type, gob_acc.offset);
    }
}


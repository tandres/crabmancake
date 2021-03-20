use crate::{error::CmcResult, graphics::Object};
use web_sys::{WebGlRenderingContext as GL, WebGlProgram};
use super::{Shader, renderable::{Renderable, BufferId, BinBuffer, ImageBuffer}, utils::*};

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

    void main() {
        vec3 normal = normalize(vNormal);
        vec3 dirLightColor = vec3(1.0, 1.0, 1.0);
        vec3 dirLightVector = vec3(-1.0, -1.0, 0.0);
        float directional = max(dot(normal, dirLightVector), 0.0);
        vec3 lighting = uAmbientLight + (directional * dirLightColor);
        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0) * vec4(lighting, 1.0);
    }
    "#;

pub(super) struct BasicShader {
    program: WebGlProgram,
}

impl BasicShader {
    pub fn new(gl: &GL) -> CmcResult<Self> {
        let program = build_program(gl, VERT_SHADER, FRAG_SHADER)?;
        Ok(Self {
            program,
        })
    }
}

impl Shader for BasicShader {
    fn renderable_update(&self, gl: &GL, renderable: &Renderable) {
    }

    fn render_objects(&self, gl: &GL, objects: &[&Object]) {
    }
}

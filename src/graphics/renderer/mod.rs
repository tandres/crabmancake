use crate::{assets::{ShaderType, AssetCache}, bus_manager::AssetMsg, error::CmcResult, Uid};
use web_sys::WebGlRenderingContext as GL;
use std::collections::{HashMap, BTreeMap};
use basic::BasicShader;
use renderable::Renderable;
use generational_arena::{Arena, Index};
use super::object::Object;

mod utils;
mod basic;
mod renderable;

use renderable::update_renderables_from_asset;

pub struct Renderer {
    shaders: HashMap<ShaderType, Box<dyn Shader>>,
    renderables: Arena<Renderable>,
}

impl Renderer {
    pub fn new(gl: &GL) -> CmcResult<Renderer> {
        let mut shaders = HashMap::new();
        shaders.insert(ShaderType::Basic, Box::new(BasicShader::new(gl)?) as Box<dyn Shader>);
        Ok( Self {
            shaders,
            renderables: Arena::new(),
        })
    }

    pub fn render(&self, gl: &GL, objects: &BTreeMap<Uid, Object>, _lights: &Vec<Uid>) {
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
        for (shader_type, shader) in self.shaders.iter() {
            let mut objects: Vec<&Object> = objects
                .iter()
                .filter(|o| &o.1.shader_type == shader_type)
                .map(|o| o.1)
                .collect();
            objects.sort_by_key(|o| o.renderable);
            shader.render_objects(gl, &objects)
        }
    }

    pub fn process_asset_msg(&mut self, gl: &GL, msg: &AssetMsg) {
        match msg {
            AssetMsg::New(name, _config) => {
                log::info!("Graphics: New Asset: {}", name);
            },
            AssetMsg::Update(name, access) => {
                AssetCache::use_asset(&access, &name, |asset| {
                    update_renderables_from_asset(asset, &mut self.renderables)
                });
                log::info!("Graphics: Asset Updated: {}", name);
            },
            AssetMsg::Complete(name, access) => {
                AssetCache::use_asset(&access, &name, |asset| {
                    update_renderables_from_asset(asset, &mut self.renderables)
                });
                log::info!("Graphics: Assets complete: {}", name);
            }
        }
    }

    fn update_shaders(&mut self, gl: &GL, changed_ones: Vec<Index>) {
        for index in changed_ones {
            if let Some(renderable) = self.renderables.get(index) {
                if let Some(shader) =self.shaders.get(renderable.get_render_target()) {
                    shader.renderable_update(gl, renderable);
                } else {
                    log::error!("Renderable has invalid shader!");
                }
            } else {
                log::error!("updated index references invalid renderable!");
            }
        }

    }
}

trait Shader {
    fn renderable_update(&self, gl: &GL, renderable: &Renderable);
    fn render_objects(&self, gl: &GL, objects: &[&Object]);
}


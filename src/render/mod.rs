use crate::{assets::Model, error::{CmcResult, CmcError}};
use gob::{Gob, GobBuffer, GobBufferTarget, GobImage};
use nalgebra::Vector3;
use std::{collections::HashMap, rc::Rc};
use web_sys::*;
use png::OutputInfo;
use gltf::mesh::Mesh;

mod shape;
mod common;
mod gob;

pub use shape::ShapeRenderer;

pub enum Light {
    Spot {
        color: Vector3<f32>,
        location: Vector3<f32>,
        direction: Vector3<f32>,
        inner_limit: f32,
        outer_limit: f32,

        intensity: f32,

        attenuator: [f32; 3],
    },
}

impl Light {
    pub fn new_point(location: [f32; 3], color: [f32; 3], intensity: f32, attenuator: [f32; 3]) -> Self {
        Self::new_spot(location, [0.; 3], color, 180.0, 180.0, intensity, attenuator)
    }

    pub fn new_spot(location: [f32; 3], pointing_at: [f32; 3], color: [f32; 3], inner_limit: f32, outer_limit: f32, intensity: f32, attenuator: [f32; 3]) -> Self {
        let location = Vector3::from(location);
        let direction = Vector3::from(pointing_at) - location;
        let color = Vector3::from(color);
        let outer_limit = f32::cos(std::f32::consts::PI * outer_limit / 180.);
        let inner_limit = f32::cos(std::f32::consts::PI * inner_limit / 180.);
        Light::Spot { location, color, direction, inner_limit, outer_limit, intensity, attenuator }
    }

}


pub struct RenderCache {
    pub shape_renderers: HashMap<String, Rc<ShapeRenderer>>,
}

impl RenderCache {
    #[allow(unused)]
    pub fn add_shaperenderer<S: AsRef<str>>(&mut self, type_name: S, renderer: ShapeRenderer) {
        let renderer = Rc::new(renderer);
        if let Some(_) = self.shape_renderers.insert(type_name.as_ref().to_string(), renderer) {
            log::warn!("Renderer for {} replaced!", type_name.as_ref());
        }
    }

    pub fn get_shaperenderer<S: AsRef<str>>(&self, type_name: S) -> Option<Rc<ShapeRenderer>> {
        self.shape_renderers.get(&type_name.as_ref().to_string()).map(|x| x.clone())
    }
}

pub fn build_rendercache(gl: &WebGlRenderingContext, models: &Vec<Model>) -> CmcResult<RenderCache> {
    let mut shape_renderers = HashMap::new();
    for model in models {
        let (gltf, buffers, images) = (&model.gltf, &model.buffers, &model.images);
        log::trace!("Gltf loaded, {} buffers and {} images", buffers.len(), images.len());
        // trace!("Gltf contents: {:?}", gltf);
        for mesh in gltf.meshes() {
            for (obj_name, renderer) in build_renderer_glb(gl, &mesh, buffers, images)? {
                if let Some(old) = shape_renderers.insert(obj_name, Rc::new(renderer)) {
                    log::warn!("Replaced renderer: {}", old.name);
                }
            }
        }
    }
    Ok(RenderCache {
        shape_renderers,
    })
}

fn build_renderer_glb(gl: &WebGlRenderingContext, object: &Mesh, buffers: &Vec<Vec<u8>>, images: &Vec<(OutputInfo, Vec<u8>)>) -> CmcResult<HashMap<String, ShapeRenderer>> {
    let name = object.name().ok_or(CmcError::missing_val("Glb mesh name")).unwrap();
    let name = format!("{}_{}", name, "glb");
    let mut cache = HashMap::new();
    let gob_buffers: Vec<GobBuffer> = buffers.iter().map(|b| GobBuffer::new(b.clone(), GobBufferTarget::Array)).collect();
    let gob_images: Vec<GobImage> = images.iter().map(|i| GobImage::from(i)).collect();
    log::debug!("Gob buffers: {}", gob_buffers.len());
    for prim in object.primitives() {
        let gob = Gob::new(&prim, &gob_buffers, &gob_images);
        if let Ok(gob) = gob {
            let renderer = ShapeRenderer::new(&name, gl, gob)?;
            cache.insert(name.clone(), renderer);
        } else {
            log::warn!("Gob build failed!");
        }
    }
    Ok(cache)
}



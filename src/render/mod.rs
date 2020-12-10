use crate::{assets::Model, error::{CmcResult, CmcError}};
use gob::{Gob, GobBuffer, GobBufferTarget, GobImage};
use std::{collections::HashMap, rc::Rc};
use web_sys::*;
use gltf::mesh::Mesh;

mod shape;
mod common;
mod gob;

pub use shape::ShapeRenderer;

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
        //log::trace!("Gltf loaded, {} buffers and {} images", buffers.len(), images.len());
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

fn build_renderer_glb(gl: &WebGlRenderingContext, object: &Mesh, buffers: &Vec<Vec<u8>>, images: &Vec<image::DynamicImage>) -> CmcResult<HashMap<String, ShapeRenderer>> {
    let name = object.name().ok_or(CmcError::missing_val("Glb mesh name")).unwrap();
    let name = format!("{}_{}", name, "glb");
    let mut cache = HashMap::new();
    let gob_buffers: Vec<GobBuffer> = buffers.iter().map(|b| GobBuffer::new(b.clone(), GobBufferTarget::Array)).collect();
    let gob_images: Vec<GobImage> = images.iter().map(|i| GobImage::from(i)).collect();
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



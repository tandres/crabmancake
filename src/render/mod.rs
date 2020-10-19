use crate::error::{CmcResult, CmcError};
use log::{error, trace, warn};
use nalgebra::{Isometry3, Perspective3, Vector3};
use std::{collections::HashMap, rc::Rc};
use web_sys::*;
use include_dir::Dir;
use wavefront_obj::obj::{Object, parse, Primitive};

mod simple;
mod shape;
mod common;

pub use simple::SimpleRenderer;
pub use shape::ShapeRenderer;
use common::CmcVertex;

pub trait Renderer {
    fn render(&self, gl: &WebGlRenderingContext, view: &Isometry3<f32>, projection: &Perspective3<f32>, location: &Vector3<f32>, rotation: &Vector3<f32>);
}

pub struct RenderCache {
    simple_renderer: SimpleRenderer,
    shape_renderers: HashMap<String, Rc<ShapeRenderer>>,
}

impl RenderCache {
    pub fn add_shaperenderer<S: AsRef<str>>(&mut self, type_name: S, renderer: ShapeRenderer) {
        let renderer = Rc::new(renderer);
        if let Some(_) = self.shape_renderers.insert(type_name.as_ref().to_string(), renderer) {
            warn!("Renderer for {} replaced!", type_name.as_ref());
        }
    }

    pub fn get_shaperenderer<S: AsRef<str>>(&self, type_name: S) -> Option<Rc<ShapeRenderer>> {
        self.shape_renderers.get(&type_name.as_ref().to_string()).map(|x| x.clone())
    }
}

pub fn build_rendercache(gl: &WebGlRenderingContext, model_dir: &Dir) -> CmcResult<RenderCache> {
    let mut shape_renderers = HashMap::new();
    let simple_renderer = SimpleRenderer::new(gl)?;
    for file in model_dir.files().iter() {
        let path = file.path();
        trace!("{} extension: {:?}", path.display(), path.extension());
        if let Some(ext) = path.extension() {
            match (ext.to_str(), file.contents_utf8()) {
                (_, None) => error!("Failed to convert file contents to utf8!"),
                (Some("obj"), Some(contents)) => {
                    for obj in parse(contents.to_string())?.objects.iter() {
                        let (obj_name, renderer) = build_renderer(gl, obj)?;
                        if let Some(old) = shape_renderers.insert(obj_name, Rc::new(renderer)) {
                            warn!("Replaced renderer: {}", old.name);
                        }
                    }
                }
                (Some(other), Some(_contents)) => warn!("Unhandled file extension {}", other),
                (None, _) => error!("Failed to convert extension to string for {:?}", file),
            }
        }
    }
    Ok(RenderCache {
        simple_renderer,
        shape_renderers,
    })
}

fn build_renderer(gl: &WebGlRenderingContext, object: &Object) -> CmcResult<(String, ShapeRenderer)> {
    let name = object.name.clone();
    let mut vertices: Vec<f32> = Vec::new();
    for vert in object.vertices.iter() {
        vertices.push(vert.x as f32);
        vertices.push(vert.y as f32);
        vertices.push(vert.z as f32);
    }

    // trace!("Object name: {}", object.name);
    // trace!("Vertices: {:?}", object.vertices.len());
    // trace!("Geometries: {:?}", object.geometry.len());
    // for geo in object.geometry.iter() {
    //     trace!("Geometry: {:#?}", geo);
    // }
    // trace!("Final vertice count {}", vertices.len());
    let mut indices: Vec<u16> = Vec::new();
    let mut normals: Vec<f32> = Vec::new();
    for geo in object.geometry.iter() {
        for shape in geo.shapes.iter() {
            match shape.primitive {
                Primitive::Triangle(a, b, c) => {
                    let missing_index = "missing normal index";
                    let out_of_range = "Normal index out of range!";
                    indices.push(a.0 as u16);
                    indices.push(b.0 as u16);
                    indices.push(c.0 as u16);
                    let index = a.2.ok_or(CmcError::missing_val(missing_index))?;
                    let normal = object.normals.get(index).ok_or(CmcError::missing_val(out_of_range))?;
                    normals.append(&mut CmcVertex::from(normal).into());
                    trace!("Triangle: A: {}({:?}) -> {}({:?})", a.0, object.vertices[a.0], index, normal);
                    let index = b.2.ok_or(CmcError::missing_val(missing_index))?;
                    let normal = object.normals.get(index).ok_or(CmcError::missing_val(out_of_range))?;
                    normals.append(&mut CmcVertex::from(normal).into());
                    trace!("          B: {}({:?}) -> {}({:?})", b.0, object.vertices[b.0], index, normal);
                    let index = c.2.ok_or(CmcError::missing_val(missing_index))?;
                    let normal = object.normals.get(index).ok_or(CmcError::missing_val(out_of_range))?;
                    normals.append(&mut CmcVertex::from(normal).into());
                    trace!("          C: {}({:?}) -> {}({:?})", c.0, object.vertices[c.0], index, normal);
                },
                _ => warn!("Unsupported primitive type!"),
            }
        }
    }

    let renderer = ShapeRenderer::new(&name, gl, vertices, indices, normals)?;
    Ok((name, renderer))
}

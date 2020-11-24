use crate::error::{CmcResult, CmcError};
use log::{error, warn};
use nalgebra::{Isometry3, Perspective3, Vector3};
use std::{collections::HashMap, rc::Rc};
use web_sys::*;
use include_dir::Dir;
use gltf::{mesh::Mesh, buffer::Data, image::Format};

mod simple;
mod shape;
mod common;

pub use simple::SimpleRenderer;
pub use shape::ShapeRenderer;


pub enum Light {
    Point {
        color: Vector3<f32>,
        location: Vector3<f32>,

        intensity: f32,
        attenuator: [f32; 3],
    },
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
        let location = Vector3::from(location);
        let color = Vector3::from(color);
        Light::Point {location, color, intensity, attenuator}
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

pub trait Renderer {
    fn render(&self, gl: &WebGlRenderingContext, view: &Isometry3<f32>, eye: &Vector3<f32>, projection: &Perspective3<f32>, location: &Vector3<f32>, rotation: &Vector3<f32>, lights: &Vec<Light>);
}

pub struct RenderCache {
    #[allow(unused)]
    simple_renderer: SimpleRenderer,
    pub shape_renderers: HashMap<String, Rc<ShapeRenderer>>,
}

impl RenderCache {
    #[allow(unused)]
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
        log::trace!("{} extension: {:?}", path.display(), path.extension());
        if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("glb") => {
                    let (gltf, buffers, images) = gltf::import_slice(file.contents())?;
                    log::trace!("Gltf loaded, {} buffers and {} images", buffers.len(), images.len());
                    // trace!("Gltf contents: {:?}", gltf);
                    for mesh in gltf.meshes() {
                        let (obj_name, renderer) = build_renderer_glb(gl, &mesh, &buffers, &images)?;
                        if let Some(old) = shape_renderers.insert(obj_name, Rc::new(renderer)) {
                            warn!("Replaced renderer: {}", old.name);
                        }
                    }
                }
                Some(other) => warn!("Unhandled file extension {}", other),
                None => error!("Failed to convert extension to string for {:?}", file),
            }
        }
    }
    // let (name, renderer) = build_test_triangle(gl)?;
    // shape_renderers.insert(name, Rc::new(renderer));
    Ok(RenderCache {
        simple_renderer,
        shape_renderers,
    })
}

// fn build_test_triangle(gl: &WebGlRenderingContext) -> CmcResult<(String, ShapeRenderer)> {
//     let test_triangle = ShapeRenderer::new(
//         &"test_triangle".to_string(),
//         gl,
//         vec![1.,1.,0.,-1.,1.,0.,-1.,-1.,0.],
//         vec![0, 1, 2],
//         vec![0.,0.,-1.,0.,0.,-1.,0.,0.,-1.],
//         vec![1.,1.,0.,-1.,1.,0.,-1.,-1.,0.])?;
//     Ok(("test_triangle".to_string(), test_triangle))
// }

fn build_renderer_glb(gl: &WebGlRenderingContext, object: &Mesh, buffers: &Vec<Data>, images: &Vec<gltf::image::Data>) -> CmcResult<(String, ShapeRenderer)> {
    let name = object.name().ok_or(CmcError::missing_val("Glb mesh name")).unwrap();
    let name = format!("{}_{}", name, "glb");
    // trace!("Name: {}", name);
    let mut out_vertices = Vec::new();
    let mut out_indices = Vec::new();
    let mut out_normals = Vec::new();
    let mut out_tex_coords = Vec::new();
    let mut out_image = Vec::new();
    let mut image_width = 0;
    let mut image_height = 0;
    for prim in object.primitives() {
        // trace!("Mode: {:?}", prim.mode());
        let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));
        if let Some(positions) = reader.read_positions() {
            for position in positions {
                // trace!("Positions: {:?}", position);
                out_vertices.extend_from_slice(&position);
            }
        }
        if let Some(indices) = reader.read_indices() {
            for index in indices.into_u32() {
                // trace!("Index: {:?}", index);
                out_indices.push(index as u16);
            }
        }
        if let Some(normals) = reader.read_normals() {
            for normal in normals {
                // trace!("Normal: {:?}", normal);
                out_normals.extend_from_slice(&normal);
            }
        }
        if let Some(texture_coordinates) = reader.read_tex_coords(0) {
            for coord in texture_coordinates.into_f32() {
                // log::trace!("Tex Coord: {:?}", coord);
                out_tex_coords.extend_from_slice(&coord);
            }
        }
        let material = prim.material();
        if let Some(texture_info) = material.pbr_metallic_roughness().base_color_texture() {
            let texture = texture_info.texture();
            let name = texture_info.texture().source().name();
            log::trace!("Image name: {:?}", name);
            log::trace!("Image index: {}", texture.index());
            let image = &images[texture.index()];
            log::trace!("Image size: {}", image.pixels.len());
            out_image = image.pixels.clone();
            image_width = image.width;
            image_height = image.height;
            log::trace!("Image format: {:?}", image.format);
            let format = match image.format {
                Format::R8G8B8A8 => WebGlRenderingContext::RGBA,
                _ => {
                    log::warn!("Format not supported!");
                    WebGlRenderingContext::RGBA
                },
            };
        }
    }
    // trace!("Indices: {} Vertices: {} Normals: {}", out_indices.len(), out_vertices.len(), out_normals.len());
    let renderer = ShapeRenderer::new(&name, gl, out_vertices, out_indices, out_normals, out_tex_coords, out_image, image_width, image_height)?;
    Ok((name, renderer))
}


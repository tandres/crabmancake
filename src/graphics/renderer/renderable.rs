use crate::error::{CmcError, CmcResult};
use crate::assets::{Asset, AssetType, ShaderType};
use gltf::{Semantic, Gltf, Node, Primitive, scene::Transform, Document};
use std::{collections::HashMap, rc::Rc};
use generational_arena::{Arena, Index};
use image::ImageFormat::{Jpeg, Png};

pub type BufferId = String;

#[derive(Debug, PartialEq, Eq)]
pub struct RenderableId {
    name: String,
    node_index: usize,
}

impl RenderableId {
    fn new<S: AsRef<str> + std::fmt::Display>(asset_name: S, node: &Node) -> RenderableId {
        let node_name = node.name().unwrap_or("unk");
        let node_index = node.index();
        let name = format!("{}{}_{}", asset_name, node_index, node_name);
        RenderableId { name, node_index }
    }
}

#[derive(Debug)]
pub struct Renderable {
    parent_asset: String,
    id: RenderableId,
    base_color: Option<Texture>,
    accessors: HashMap<Attribute, Accessor>,
    indices: Option<Accessor>,
    base_transform: [[f32; 4]; 4],
    render_target: ShaderType,
}

impl Renderable {
    fn new(asset: &Asset, id: RenderableId) -> Self {
        Renderable {
            parent_asset: asset.name().to_string(),
            id,
            base_color: None,
            accessors: HashMap::new(),
            indices: None,
            base_transform: [[0.; 4]; 4],
            render_target: asset.get_config().render_type.clone(),
        }
    }

    pub fn get_render_target<'a>(&'a self) -> &'a ShaderType {
        &self.render_target
    }

    fn update(&mut self, gltf: &Gltf, asset: &Asset, cache: &mut BufferCache) -> CmcResult<()> {
        log::info!("{}: Updating", self.id.name);
        let node = gltf.nodes().nth(self.id.node_index).expect("Failed to find node index");
        let mesh = node.mesh().expect("No mesh for node!");
        let prim = if mesh.primitives().count() > 1 {
            log::warn!("{}: Mesh had multiple primitives, this isn't supported yet!", self.id.name);
            mesh.primitives().next()
        } else if mesh.primitives().count() == 0 {
            log::error!("Mesh had no primitives!");
            None
        } else {
            mesh.primitives().next()
        }.unwrap();
        for (sem, gacc) in prim.attributes() {
            let attr = Attribute::from(&sem);
            if !attr.is_supported() {
                continue;
            }
            let mut acc = self.accessors.entry(attr.clone()).or_insert(Accessor::new(attr, &gacc, asset.name()));
            match (&acc.buffer, cache.get_bin(&acc.buffer_id)) {
                (Some(_), _) => (),
                (None, Some(b)) => acc.buffer = Some(b),
                (None, None) => {
                    match cache.lookup_bin_from_asset(&acc.buffer_id, gltf, asset) {
                        Ok(val) => acc.buffer = Some(val),
                        Err(CmcError::NotYet) => {
                            if asset.is_complete() {
                                Err(CmcError::missing_val("asset is complete but missing component"))?
                            }
                        },
                        Err(e) => Err(e)?,
                    }
                }
            }
        }
        if let None = self.indices {
            log::trace!("{}: Indices empty, setting accessor", self.id.name);
            let accr = prim.indices().ok_or(CmcError::missing_val("Prim missing indices"))?;
            let attr = Attribute::Indices;
            let mut accr = Accessor::new(attr, &accr, asset.name());
            accr.buffer_id = build_buffer_id(BufferSource::Index(node.index()), asset.name());
            self.indices = Some(accr);
        }
        let indices = if let Some(indices) = self.indices.take() {
            if indices.buffer.is_some() {
                Some(indices)
            } else {
                log::info!("{}: Populating index buffer", self.id.name);
                let accr = prim.indices().ok_or(CmcError::missing_val("Prim missing indices"))?;
                let view = accr.view().ok_or(CmcError::missing_val("Buffer view missing"))?;
                let source_buffer_id = build_buffer_id(view.buffer().source(), asset.name());
                let buffer = if let Some(bin) = cache.get_bin(&source_buffer_id) {
                    let data = bin.data[view.offset()..(view.offset() + view.length())].to_vec();
                    Some(Rc::new(BinBuffer { id: indices.buffer_id.clone(), data }))
                } else {
                    log::info!("{}: Binary file {} not yet loaded", self.id.name, source_buffer_id);
                    None
                };
                Some(Accessor{ buffer, ..indices })
            }
        } else {
            None
        };

        self.indices = indices;

        if let None = self.base_color {
            log::info!("{}: Base color texture empty, populating", self.id.name);
            let texture_info = prim.material().pbr_metallic_roughness().base_color_texture().ok_or(CmcError::missing_val("No texture info for base color"))?;
            let texture = texture_info.texture();
            let source = texture.source().source();
            let sampler = texture.sampler();
            let target_id = build_buffer_id(BufferSource::Image(source), asset.name());
            let status = TextureStatus::Pending(target_id);
            let wrap_s = sampler.wrap_s().as_gl_enum();
            let wrap_t = sampler.wrap_t().as_gl_enum();
            self.base_color = Some(Texture { status, wrap_s, wrap_t });
        }
        let base_color = if let Some(base_color) = self.base_color.take() {
            let status = match base_color.status {
                TextureStatus::Available(buf) => {
                    log::info!("{}: Texture already loaded", self.id.name);
                    TextureStatus::Available(buf)
                },
                TextureStatus::Pending(target) => {
                    log::info!("{}: Texture is pending", self.id.name);
                    if let Some(img) = cache.get_image(&target) {
                        log::info!("Texture was in cache");
                        TextureStatus::Available(img)
                    } else {
                        match cache.lookup_img_from_asset(&target, gltf, asset) {
                            Ok(img) => {
                                log::info!("{}: Was able to read texture", self.id.name);
                                TextureStatus::Available(img)
                            },
                            Err(CmcError::NotYet) => {
                                if asset.is_complete() {
                                    Err(CmcError::missing_val(format!("{}: asset is complete but missing component", self.id.name)))?
                                } else {
                                    log::info!("{}: Asset {} not yet available", self.id.name, target);
                                    TextureStatus::Pending(target)
                                }
                            }
                            Err(e) => Err(e)?,
                        }
                    }
                },
                TextureStatus::Unused => TextureStatus::Unused,
            };
            Some(Texture{ status, ..base_color })
        } else {
            None
        };
        self.base_color = base_color;
        Ok(())
    }
}

struct BufferCache {
    bin: HashMap<BufferId, Rc<BinBuffer>>,
    image: HashMap<BufferId, Rc<ImageBuffer>>,
}

impl BufferCache {
    fn get_image(&mut self, id: &BufferId) -> Option<Rc<ImageBuffer>> {
        self.image.get(id).map(|i| i.clone())
    }

    fn lookup_img_from_asset(&mut self, id: &BufferId, gltf: &Gltf, asset: &Asset) -> CmcResult<Rc<ImageBuffer>> {
        let gltf_image = gltf.images().find(|i| {
            *id == build_buffer_id(i.source(), asset.name())
        }).ok_or(CmcError::missing_val("ImageBuffer not found in gltf!"))?;
        use gltf::image::Source;
        let (raw_data, mime) = match gltf_image.source() {
            Source::View { view, mime_type } => {
                log::info!("BufferView image: {:?}", view.buffer().index());
                let (id, bin) = self.bin.iter().next().ok_or(CmcError::missing_val("No binaries in cache for binary image source!"))?;
                if *id != build_buffer_id(gltf_image.source(), asset.name()) {
                    Err(CmcError::missing_val("Binary source buffer doesn't match!"))?;
                }
                let buf_data = bin.data[view.offset()..(view.offset() + view.length())].to_vec();
                (buf_data, Some(mime_type))
            },
            Source::Uri { uri, mime_type } => {
                log::info!("Uri image: {}", uri);
                let file = asset.get_file(uri).ok_or(CmcError::NotYet)?;
                (file.clone(), mime_type)
            }
        };

        let format = match mime {
            Some("image/jpeg") => Jpeg,
            Some("image/png") => Png,
            _ => panic!("Unknown image format!"),
        };
        let dyn_image = image::load_from_memory_with_format(&raw_data, format)?;
        let data = dyn_image.into_rgba8();
        let (width, height) = data.dimensions();
        let img = Rc::new(ImageBuffer {id: id.clone(), width, height, data: data.into_raw()});
        self.add_image(id, img.clone());
        Ok(img.clone())
    }

    fn add_image(&mut self, id: &BufferId, img: Rc<ImageBuffer>) {
        if self.image.insert(id.to_string(), img).is_some() {
            log::warn!("BufferCache image entry for {} overwritten", id);
        }
    }

    fn get_bin(&mut self, id: &BufferId) -> Option<Rc<BinBuffer>> {
        self.bin.get(id).map(|b| b.clone())
    }

    fn lookup_bin_from_asset(&mut self, id: &BufferId, gltf: &Gltf, asset: &Asset) -> CmcResult<Rc<BinBuffer>> {
        let buffer = gltf.buffers().find(|b| {
            *id == build_buffer_id(BufferSource::from(b.source()), asset.name())
        }).ok_or(CmcError::missing_val("BinaryBuffer not found in gltf!"))?;
        use gltf::buffer::Source;
        let bin = Rc::new(match buffer.source() {
            Source::Bin => {
                let blob = gltf.blob.as_ref().ok_or(CmcError::missing_val("Missing gltf blob!"))?;
                let mut data = blob.clone();
                while data.len() % 4 != 0 {
                    data.push(0);
                }
                BinBuffer { id: id.clone(), data }
            },
            Source::Uri(file_name) => {
                log::info!("looking for {} in asset", file_name);
                let file = asset.get_file(file_name).ok_or(CmcError::NotYet)?;
                let data = file.clone();
                BinBuffer { id: id.clone(), data }
            }
        });
        self.add_bin(id, bin.clone());
        Ok(bin.clone())
    }

    fn add_bin(&mut self, id: &BufferId, bin: Rc<BinBuffer>) {
        if self.bin.insert(id.to_string(), bin).is_some() {
            log::warn!("BufferCache bin entry for {} overwritten!", id);
        }
    }
}

pub fn update_renderables_from_asset(asset: &Asset, renderables: &mut Arena<Renderable>) -> CmcResult<Vec<Index>> {
    let gltf = get_gltf(asset)?;
    let mut renderable_ids = Vec::new();
    for node in gltf.nodes() {
        renderable_ids.push(RenderableId::new(asset.name(), &node));
    }
    let new_renderables = renderable_ids.into_iter().filter(|n| {
        renderables.iter().find(|r| r.1.id.name == n.name).is_none()
    })
        .map(|n| Renderable::new(asset, n))
        .collect::<Vec<Renderable>>();
    log::info!("{} New renderables from asset", new_renderables.len());
    for renderable in new_renderables.into_iter() {
        renderables.insert(renderable);
    }
    let mut buffer_cache = BufferCache { bin: HashMap::new(), image: HashMap::new() };
    for (_, renderable) in renderables.iter_mut().filter(|r| r.1.parent_asset == asset.name()) {
        let _ = renderable.update(&gltf, asset, &mut buffer_cache)?;
        log::info!("{:?}", renderable);
    }
    let updated_shaders: Vec<Index> = renderables
        .iter()
        .filter(|r| r.1.parent_asset == asset.name())
        .map(|r| r.0.clone())
        .collect();
    Ok(updated_shaders)
}

fn get_gltf(asset: &Asset) -> CmcResult<Gltf> {
    let config = asset.get_config();
    let gl_root = match config.asset_type {
        AssetType::GltfModel { ref gl_root, prompt_files: _, deferrable_files: _ } => {
            gl_root
        }
        AssetType::GlbModel { ref gl_root } => {
            gl_root
        }
    };
    let file = asset.get_file(gl_root).ok_or(CmcError::missing_val("No gl root file in asset!"))?;
    Ok(Gltf::from_slice(file)?)
}

#[derive(Debug)]
pub struct ImageBuffer {
    id: BufferId,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

#[derive(Debug)]
pub struct BinBuffer {
    id: BufferId,
    data: Vec<u8>,
}


enum BufferSource<'a> {
    Binary(gltf::buffer::Source<'a>),
    Image(gltf::image::Source<'a>),
    Index(usize),
}

impl<'a> From<gltf::buffer::Source<'a>> for BufferSource<'a> {
    fn from(source: gltf::buffer::Source<'a>) -> Self {
        BufferSource::Binary(source)
    }
}

impl<'a> From<gltf::image::Source<'a>> for BufferSource<'a> {
    fn from(source: gltf::image::Source<'a>) -> Self {
        BufferSource::Image(source)
    }
}

fn build_buffer_id<'a, T: Into<BufferSource<'a>>>(src: T, asset_name: &str) -> BufferId {
    let src: BufferSource = src.into();
    match src {
        BufferSource::Binary(gltf::buffer::Source::Bin) => {
            format!("{}bbb", asset_name)
        },
        BufferSource::Binary(gltf::buffer::Source::Uri(path)) => {
            format!("{}bbu{}", asset_name, path)
        },
        BufferSource::Image(gltf::image::Source::View {view, mime_type: _}) => {
            format!("{}biv{}", asset_name, view.name().unwrap_or("unk"))
        },
        BufferSource::Image(gltf::image::Source::Uri {uri, mime_type: _}) => {
            format!("{}biu{}", asset_name, uri)
        },
        BufferSource::Index(node_index) => {
            format!("{}ind{}", asset_name, node_index)
        }
    }
}

#[derive(Debug)]
enum TextureStatus {
    Pending(BufferId),
    Available(Rc<ImageBuffer>),
    Unused,
}

#[derive(Debug)]
struct Texture {
    status: TextureStatus,
    wrap_s: u32,
    wrap_t: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Attribute {
    Positions,
    TexCoords(u32),
    Normals,
    Unhandled,
    Indices,
}

impl From<&Semantic> for Attribute {
    fn from(semantic: &Semantic) -> Self {
        match semantic {
            Semantic::Positions => Attribute::Positions,
            // Semantic::Extras(_name) => Attribute::Unhandled,
            Semantic::Normals => Attribute::Normals,
            Semantic::Tangents => Attribute::Unhandled,
            Semantic::Colors(_index) => Attribute::Unhandled,
            Semantic::TexCoords(index) => Attribute::TexCoords(*index),
            Semantic::Joints(_index) => Attribute::Unhandled,
            Semantic::Weights(_index) => Attribute::Unhandled,
        }
    }
}

impl Attribute {
    fn is_supported(&self) -> bool {
        *self != Attribute::Unhandled
    }
}

#[derive(Debug)]
struct Accessor {
    pub attribute: Attribute,
    pub buffer: Option<Rc<BinBuffer>>,
    pub buffer_id: BufferId,
    pub data_type: u32,
    pub stride: i32,
    pub count: usize,
    pub num_items: i32,
    pub normalized: bool,
    pub offset: i32,
}

impl Accessor {
    fn new(attribute: Attribute, accessor: &gltf::Accessor, asset_name: &str) -> Self {
        let view = accessor.view().unwrap();
        let stride = view.stride().unwrap_or(0) as i32;
        let num_items = accessor.dimensions().multiplicity() as i32;
        let offset = view.offset() as i32;
        let buffer_id = build_buffer_id(view.buffer().source(), asset_name);
        Self {
            attribute,
            buffer: None,
            buffer_id,
            count: accessor.count(),
            data_type: accessor.data_type().as_gl_enum(),
            stride,
            num_items,
            normalized: accessor.normalized(),
            offset,
        }
    }
}


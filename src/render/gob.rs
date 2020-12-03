use crate::{error::{CmcResult, CmcError}};
use std::collections::HashMap;
use gltf::{mesh::{Primitive, Semantic}, accessor::{Accessor, DataType}};

#[derive(Debug)]
pub struct Gob {
    pub accessors: HashMap<GobDataAttribute, GobDataAccess>,
    pub buffers: HashMap<usize, GobBuffer>,
    //image_info: Somethinghere
    pub images: HashMap<usize, GobImage>,
}

impl Gob {
    pub fn new(primitive: &Primitive, avail_buffers: &Vec<GobBuffer>) -> CmcResult<Gob> {
    let mut accessors = HashMap::new();
    let mut gob_buffers = HashMap::new();
    for (sem, attr) in primitive.attributes() {
        let gob_attribute = GobDataAttribute::from(&sem);
        if let GobDataAttribute::Unhandled = gob_attribute {
            log::warn!("Semantic: {:?} unhandled", sem);
            continue;
        }
        let acc = GobDataAccess::from_accessor(&sem, &attr);
        let buffer_index = acc.buffer_index;
        if !gob_buffers.contains_key(&buffer_index) {
            if avail_buffers.len() <= buffer_index {
                log::error!("Buffer index not present in available buffers!");
                Err(CmcError::missing_val("Missing buffer index!"))?;
            }
            gob_buffers.insert(acc.buffer_index, avail_buffers[buffer_index].clone());
        }
        accessors.insert(gob_attribute, acc);
    }
    if let Some(index_acc) = primitive.indices() {
        let mut attr = GobDataAccess::new(GobDataAttribute::Indices, &index_acc);
        let offset = attr.offset as usize;
        let size = index_acc.view().ok_or(CmcError::missing_val("No view for index accessor"))?.length();
        if avail_buffers.len() <= attr.buffer_index {
            log::error!("No matching buffer for indices");
            Err(CmcError::missing_val("Missing buffer index"))?;
        }
        let copied_data = avail_buffers[attr.buffer_index].copy_from_buffer(offset, size)?;
        let new_gob_buffer = GobBuffer::new(copied_data, GobBufferTarget::ElementArray);
        gob_buffers.insert(std::usize::MAX, new_gob_buffer);
        attr.buffer_index = std::usize::MAX;
        attr.offset = 0;
        accessors.insert(GobDataAttribute::Indices, attr);
    }

    Ok(Gob {
        accessors,
        buffers: gob_buffers,
        images: HashMap::new()
    })
}

}

#[derive(Clone, Debug)]
pub enum GobBufferTarget {
    Array,
    ElementArray,
}

impl GobBufferTarget {
    pub fn to_gl(&self) -> u32 {
        use web_sys::WebGlRenderingContext as GL;
        match self {
            Self::Array => GL::ARRAY_BUFFER,
            Self::ElementArray => GL::ELEMENT_ARRAY_BUFFER,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GobBuffer {
    pub data: Vec<u8>,
    pub target: GobBufferTarget,
}

impl GobBuffer {
    pub fn new(data: Vec<u8>, target: GobBufferTarget) -> Self {
        Self { data, target }
    }

    pub fn copy_from_buffer(&self, offset: usize, bytes: usize) -> CmcResult<Vec<u8>> {
        if self.data.len() < offset + bytes {
            log::error!("A copy from buffer starting at {} for {} bytes failed: Buffer too small {}", offset, bytes, self.data.len());
            Err(CmcError::missing_val("Buffer not large enough to copy from"))?
        }
        Ok(self.data[offset..(offset + bytes)].to_vec())
    }
}

#[derive(Debug)]
pub struct GobImage {
    pub image_height: u32,
    pub image_width: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum GobDataAttribute {
    Positions,
    TexCoords(u32),
    Normals,
    Unhandled,
    Indices,
}

impl From<&Semantic> for GobDataAttribute {
    fn from(semantic: &Semantic) -> Self {
        match semantic {
            Semantic::Positions => GobDataAttribute::Positions,
            // Semantic::Extras(_name) => GobDataAttribute::Unhandled,
            Semantic::Normals => GobDataAttribute::Normals,
            Semantic::Tangents => GobDataAttribute::Unhandled,
            Semantic::Colors(_index) => GobDataAttribute::Unhandled,
            Semantic::TexCoords(index) => GobDataAttribute::TexCoords(*index),
            Semantic::Joints(_index) => GobDataAttribute::Unhandled,
            Semantic::Weights(_index) => GobDataAttribute::Unhandled,
        }
    }
}

#[derive(Debug)]
pub struct GobDataAccess {
    pub attribute: GobDataAttribute,
    pub buffer_index: usize,
    pub data_type: u32,
    pub stride: i32,
    pub count: usize,
    pub num_items: i32,
    pub normalized: bool,
    pub offset: i32,
    pub gl_attribute_index: Option<u32>,
}

impl GobDataAccess {
    fn from_accessor(semantic: &Semantic, accessor: &Accessor) -> Self {
        Self::new(semantic.into(), accessor)
    }

    fn new(attribute: GobDataAttribute, accessor: &Accessor) -> Self {
        let view = accessor.view().unwrap();
        let buffer = view.buffer();
        let buffer_index = buffer.index();
        let stride = view.stride().unwrap_or(0) as i32;
        let num_items = accessor.dimensions().multiplicity() as i32;
        let offset = view.offset() as i32;
        Self {
            attribute,
            buffer_index,
            count: accessor.count(),
            data_type: gltf_type_to_gl_type(accessor.data_type()),
            stride,
            num_items,
            normalized: accessor.normalized(),
            offset,
            gl_attribute_index: None,
        }
    }
}

fn gltf_type_to_gl_type(input: DataType) -> u32 {
    use web_sys::WebGlRenderingContext as GL;
    use DataType::*;
    match input {
        I8 => GL::BYTE,
        U8 => GL::UNSIGNED_BYTE,
        I16 => GL::SHORT,
        U16 => GL::UNSIGNED_SHORT,
        U32 => GL::UNSIGNED_INT,
        F32 => GL::FLOAT,
    }
}

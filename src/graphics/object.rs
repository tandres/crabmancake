use nphysics3d::nalgebra::Isometry3;
use generational_arena::Index;
use crate::assets::ShaderType;

pub struct Object {
    pub renderable: Index,
    pub shader_type: ShaderType,
    pub position: Isometry3<f32>,
}

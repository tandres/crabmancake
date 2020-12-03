use nalgebra::{Isometry3, Perspective3, Vector3};
use crate::render::Light;

pub struct Scene {
    view: Isometry3<f32>,
    eye: Vector3<f32>,
    projection: Perspective3<f32>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn new(view: Isometry3<f32>, eye: Vector3<f32>, projection: Perspective3<f32>, lights: Vec<Light>) -> Self {
        Self {
            view, eye, projection, lights
        }
    }

    pub fn get_view_as_vec(&self) -> Vec<f32> {
        self.view.to_homogeneous().as_slice().to_vec()
    }

    pub fn get_eye_as_vec(&self) -> Vec<f32> {
        self.eye.to_homogeneous().as_slice().to_vec()
    }

    pub fn get_projection_as_vec(&self) -> Vec<f32> {
        self.projection.to_homogeneous().as_slice().to_vec()
    }
}

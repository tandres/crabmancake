use crate::{light::Light, render::ShapeRenderer, scene::Scene};
use nphysics3d::nalgebra::Isometry3;
use web_sys::WebGlRenderingContext;
use std::rc::Rc;

pub struct Shape {
    renderer: Rc<ShapeRenderer>,
    // be cool to figure out how pre-computing the transforms on rotation and translation
    // might be more efficient. Super early to answer but interesting thought.
    // For now just dumping everything into entity then we'll move it into a phys from there.
    // Way to think about optimizing way too early.
    pub position: Isometry3<f32>,
}

impl Shape {
    pub fn new(renderer: Rc<ShapeRenderer>, position: Isometry3<f32>) -> Self {
        Self { renderer, position }
    }

    pub fn render(&self, gl: &WebGlRenderingContext, scene: &Scene, lights: &Vec<Light>) {
        self.renderer.render(gl, scene, lights, &self.position)
    }
}

use crate::{light::Light, render::ShapeRenderer, entity::Entity, scene::Scene};
use web_sys::WebGlRenderingContext;
use std::rc::Rc;

pub struct Shape {
    renderer: Rc<ShapeRenderer>,
    // be cool to figure out how pre-computing the transforms on rotation and translation
    // might be more efficient. Super early to answer but interesting thought.
    // For now just dumping everything into entity then we'll move it into a phys from there.
    // Way to think about optimizing way too early.
    pub entity: Entity,
}

impl Shape {
    pub fn new(renderer: Rc<ShapeRenderer>, entity: Entity) -> Self {
        Self { renderer, entity }
    }

    pub fn render(&self, gl: &WebGlRenderingContext, scene: &Scene, lights: &Vec<Light>) {
        self.renderer.render(gl, scene, lights, &self.entity.location, &self.entity.rotation)
    }
}

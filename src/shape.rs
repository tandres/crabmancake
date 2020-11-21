use crate::{render::{Light, Renderer}, entity::Entity};
use nalgebra::{Isometry3, Perspective3, Vector3};
use web_sys::WebGlRenderingContext;
use std::rc::Rc;

pub struct Shape<R>
where
    R: Renderer,
{
    renderer: Rc<R>,
    // be cool to figure out how pre-computing the transforms on rotation and translation
    // might be more efficient. Super early to answer but interesting thought.
    // For now just dumping everything into entity then we'll move it into a phys from there.
    // Way to think about optimizing way too early.
    pub entity: Entity,
}

impl<R> Shape<R>
where
    R: Renderer,
{
    pub fn new(renderer: Rc<R>, entity: Entity) -> Self {
        Self { renderer, entity }
    }

    pub fn render(&self, gl: &WebGlRenderingContext, view: &Isometry3<f32>, eye: &Vector3<f32>, projection: &Perspective3<f32>, lights: &Vec<Light>) {
        self.renderer.render(gl, view, eye, projection, &self.entity.location, &self.entity.rotation, lights)
    }


}

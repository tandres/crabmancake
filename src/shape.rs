use crate::{render::Renderer, entity::Entity};
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

    pub fn render(&self, gl: &WebGlRenderingContext, canvas_height: f32, canvas_width: f32) {
        self.renderer.render(gl, canvas_height, canvas_width, &self.entity.location, &self.entity.rotation)
    }
}

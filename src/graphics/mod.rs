use crate::{Uid, bus::Receiver, bus_manager::*, error::CmcResult};
use std::{collections::BTreeMap, rc::Rc};

mod canvas;
mod renderer;
mod object;
mod camera;
mod light;

use renderer::Renderer;
use object::Object;

pub struct Graphics {
    asset_rx: Receiver<AssetMsg>,
    render_rx: Receiver<RenderMsg>,
    canvas: canvas::Canvas,
    renderer: renderer::Renderer,
    objects: BTreeMap<Uid, Object>,
    lights: Vec<Uid>,
}

impl Graphics {
    pub fn new(bus_manager: &Rc<BusManager>) -> CmcResult<Self> {
        let canvas = canvas::Canvas::new(
            "rustCanvas",
            "canvasSide",
            bus_manager.render.new_sender());
        let renderer = Renderer::new(canvas.get_gl())?;
        Ok( Self {
            asset_rx: bus_manager.asset.new_receiver(),
            render_rx: bus_manager.render.new_receiver(),
            canvas,
            renderer,
            objects: BTreeMap::new(),
            lights: Vec::new(),
        })
    }

    pub fn update(&mut self, _timestep: f64) {
        self.process_asset_msgs();
        self.process_render_msgs();
        self.render();
    }

    fn render(&self) {
        self.renderer.render(self.canvas.get_gl(), &self.objects, &self.lights);
    }

    fn process_asset_msgs(&mut self) {
        for event in self.asset_rx.read() {
            self.renderer.process_asset_msg(self.canvas.get_gl(), event.as_ref());
        }
    }

    fn process_render_msgs(&mut self) {
        for event in self.render_rx.read() {
            match event.as_ref() {
                RenderMsg::Resize(width, height) => {
                    let gl = self.canvas.get_gl();
                    gl.viewport(0, 0, *width as i32, *height as i32);
                },
                _ => (),
            }
        }
    }
}

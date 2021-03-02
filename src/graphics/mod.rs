use crate::{bus::Receiver, bus_manager::*, assets::AssetCache};
use std::rc::Rc;

mod graphics_object;
mod canvas;

pub use graphics_object::GraphicsObject;

pub struct Graphics {
    asset_rx: Receiver<AssetMsg>,
    render_rx: Receiver<RenderMsg>,
    canvas: canvas::Canvas,
}

impl Graphics {
    pub fn new(bus_manager: &Rc<BusManager>) -> Self {
        let canvas = canvas::Canvas::new(
            "rustCanvas",
            "canvasSide",
            bus_manager.render.new_sender());
        Self {
            asset_rx: bus_manager.asset.new_receiver(),
            render_rx: bus_manager.render.new_receiver(),
            canvas,
        }
    }

    pub fn update(&mut self, _timestep: f64) {
        self.process_asset_msgs();
        self.process_render_msgs();
        self.render();
    }

    fn render(&self) {
        use web_sys::WebGlRenderingContext as GL;
        self.canvas.get_gl().clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
    }

    fn process_asset_msgs(&mut self) {
        for event in self.asset_rx.read() {
            match event.as_ref() {
                AssetMsg::New(name, _config) => {
                    log::info!("New Asset: {}", name);
                },
                AssetMsg::Update(name, _access) => {
                    log::info!("Asset Updated: {}", name);
                },
                AssetMsg::Complete(name, access) => {
                    let config = AssetCache::use_asset(&access, &name, |a| a.get_config().clone());
                    let asset_info = AssetCache::use_asset(&access, &name, |a| a.get_asset_info());
                    log::info!("Graphics: Asset Complete: {} {:?}", name, config);
                    log::info!("Graphics: Asset file info: {:?}", asset_info);
                }
            }
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

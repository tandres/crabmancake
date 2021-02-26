use crate::{bus::Receiver, bus_manager::*, assets::AssetCache};
use std::rc::Rc;

mod graphics_object;

pub use graphics_object::GraphicsObject;

pub struct Graphics {
    receiver: Receiver<AssetMsg>,
}

impl Graphics {
    pub fn new(bus_manager: &Rc<BusManager>) -> Self {
        Self {
            receiver: bus_manager.asset.new_receiver(),
        }
    }

    pub fn update(&mut self, _timestep: f64) {
        self.process_asset_msgs();
    }

    fn process_asset_msgs(&mut self) {
        for event in self.receiver.read() {
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
                    log::info!("Asset Complete: {} {:?}", name, config);
                    log::info!("Asset file info: {:?}", asset_info);
                }
            }
        }
    }
}

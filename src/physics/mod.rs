use crate::{bus::Receiver, bus_manager::*, assets::AssetCache};
use std::rc::Rc;

mod physics_object;

pub use physics_object::PhysicsObject;

pub struct Physics {
    receiver: Receiver<AssetMsg>,
}

impl Physics {
    pub fn new(bus_manager: &Rc<BusManager>) -> Self {
        Self {
            receiver: bus_manager.asset.new_receiver(),
        }
    }

    pub fn update(&self, _timestep: f64) {
        let asset_events = self.receiver.read();
        for event in asset_events {
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
                    log::info!("Physics: Asset Complete: {} {:?}", name, config);
                    log::info!("Physics: Asset file info: {:?}", asset_info);
                }
            }
        }
    }
}

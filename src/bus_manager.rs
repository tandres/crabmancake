use crate::assets::Model;
use crate::bus::{Bus, create_bus};
use std::rc::Rc;
use crate::{assets::{Config, AssetCacheAccess}, uid::Uid};
use nphysics3d::nalgebra::Isometry3;

pub enum AssetMsg {
    New(String, Config),
    Update(String, AssetCacheAccess),
    Complete(String, AssetCacheAccess),
}

pub enum RenderMsg {
    NewModel(Rc<Model>),
    NewObject(Uid, String, [f32; 3]),
    SetTarget(Uid),
    ObjectUpdate(Uid, Isometry3<f32>),
}

pub enum UiMsg {
    NewObject(Uid, [f32; 3]),
    SetTarget(Uid),
}

#[derive(Clone)]
pub struct BusManager {
    id: u32,
    pub render: Bus<RenderMsg>,
    pub ui: Bus<UiMsg>,
    pub asset: Bus<AssetMsg>,
}

impl BusManager {
    pub fn new(id: u32) -> Self {
        BusManager {
            id,
            render : create_bus(),
            ui : create_bus(),
            asset: create_bus(),
        }
    }
}

impl PartialEq for BusManager {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

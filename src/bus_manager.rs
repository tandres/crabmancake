use crate::{scene::Scene, shape::Shape, assets::Model, light::{Attenuator, Light}};
use crate::bus::{Bus, create_bus};
use std::rc::Rc;
use crate::uid::Uid;

pub enum RenderMsg {
    NewModel(Rc<Model>),
    NewObject(Uid, String),
}

pub enum UiMsg {
    NewObject(Uid),
}

pub struct BusManager {
    id: u32,
    pub render: Bus<RenderMsg>,
    pub ui: Bus<UiMsg>,
}

impl BusManager {
    pub fn new(id: u32) -> Self {
        BusManager {
            id,
            render : create_bus(),
            ui : create_bus()
        }
    }
}

impl PartialEq for BusManager {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

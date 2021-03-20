#![recursion_limit="256"]
use crate::{assets::AssetCache, bus::{Sender, Receiver}};
use crate::bus_manager::*;
use log::trace;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::Document;
use std::rc::Rc;
use error::CmcError;
use std::collections::HashMap;
use nphysics3d::nalgebra::Vector3;
use nphysics3d::ncollide3d::shape::{Cuboid, ShapeHandle};
use nphysics3d::force_generator::DefaultForceGeneratorSet;
use nphysics3d::joint::DefaultJointConstraintSet;
use nphysics3d::object::{
    BodyPartHandle, ColliderDesc, DefaultBodySet, DefaultColliderSet, Ground, RigidBody, RigidBodyDesc,
};
use nphysics3d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};
use generational_arena::Index;

const GIT_VERSION: &str = git_version::git_version!();

mod control_panel;
mod bus;
mod bus_manager;
mod key_state;
mod error;
mod assets;
mod uid;
mod physics;
mod graphics;

pub use uid::Uid;

#[wasm_bindgen]
pub struct CmcClient {
    asset_cache: AssetCache,
    ui_receiver: Receiver<UiMsg>,
    render_sender: Sender<RenderMsg>,
    last_time: f64,
    handle_uid_lut: HashMap<Index, Uid>,
    mechanical_world: DefaultMechanicalWorld<f32>,
    geometrical_world: DefaultGeometricalWorld<f32>,
    bodies: DefaultBodySet<f32>,
    colliders: DefaultColliderSet<f32>,
    joint_constraints: DefaultJointConstraintSet<f32>,
    force_generators: DefaultForceGeneratorSet<f32>,
    physics: physics::Physics,
    graphics: graphics::Graphics,
}

#[wasm_bindgen]
impl CmcClient {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<CmcClient, JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let document: Document = window.document().expect("should have a document on window");
        let bus_manager = Rc::new(BusManager::new(0));
        let physics = physics::Physics::new(&bus_manager);
        let graphics = graphics::Graphics::new(&bus_manager)?;
        let canvas_side = document.get_element_by_id("canvasSide").ok_or(CmcError::missing_val("canvasSide"))?;
        let mechanical_world = DefaultMechanicalWorld::new(Vector3::new(0.0, -9.81, 0.0));
        let geometrical_world = DefaultGeometricalWorld::new();
        let mut bodies = DefaultBodySet::new();
        let mut colliders = DefaultColliderSet::new();
        let joint_constraints = DefaultJointConstraintSet::new();
        let force_generators = DefaultForceGeneratorSet::new();

        let ground_thickness = 0.2;
        let ground_width = 3.0;
        let ground_shape = ShapeHandle::new(Cuboid::new(Vector3::new(
            ground_width,
            ground_thickness,
            ground_width,
        )));

        let asset_cache = AssetCache::new(bus_manager.clone());
        assets::start_asset_fetch(&asset_cache);
        let ground_handle = bodies.insert(Ground::new());
        let co = ColliderDesc::new(ground_shape)
            .translation(Vector3::y() * -ground_thickness)
            .build(BodyPartHandle(ground_handle, 0));
        colliders.insert(co);

        let panel = document.get_element_by_id("controlPanel").ok_or(CmcError::missing_val("controlPanel"))?;
        control_panel::ControlPanelModel::mount(&panel, control_panel::ControlPanelProps { bus_manager: bus_manager.clone()});
        let render_sender = bus_manager.render.new_sender();

        let client = CmcClient {
            asset_cache: asset_cache,
            last_time: js_sys::Date::now(),
            ui_receiver: bus_manager.ui.new_receiver(),
            handle_uid_lut: HashMap::new(),
            render_sender,
            mechanical_world,
            geometrical_world,
            bodies,
            colliders,
            joint_constraints,
            force_generators,
            physics,
            graphics,
        };
        Ok(client)
    }

    pub fn update(&mut self) -> Result<(), JsValue> {
        let time = js_sys::Date::now();
        let delta_t =  time - self.last_time;
        self.last_time = time;
        self.physics.update(delta_t);
        self.graphics.update(delta_t);
        let ui_events = self.ui_receiver.read();
        for event in ui_events {
            match event.as_ref() {
                UiMsg::NewObject(uid, position) => {
                    let cuboid = ShapeHandle::new(Cuboid::new(Vector3::repeat(1.0)));
                    let rb = RigidBodyDesc::new()
                        .translation(Vector3::new(position[0], position[1], position[2]))
                        .build();
                    let rb_handle = self.bodies.insert(rb);

                    // Build the collider.
                    let co = ColliderDesc::new(cuboid)
                        .density(1.0)
                        .build(BodyPartHandle(rb_handle.clone(), 0));
                    self.colliders.insert(co);
                    log::info!("Added new object: {:?}", rb_handle);
                    self.handle_uid_lut.insert(rb_handle, uid.clone());
                    self.render_sender.send(RenderMsg::NewObject(uid.clone(), "Ground_glb".to_string(), *position));
                },
                UiMsg::SetTarget(uid) => {
                    self.render_sender.send(RenderMsg::SetTarget(uid.clone()))
                },
            }
        }
        self.mechanical_world.set_timestep((delta_t / 1000.0) as f32);
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators);
        for (handle, body) in self.bodies.iter() {
            if let Some(rigid) = body.downcast_ref::<RigidBody<f32>>() {
                let pos = rigid.position();

                if let Some(uid) = self.handle_uid_lut.get(&handle) {
                    self.render_sender.send(RenderMsg::ObjectUpdate(uid.clone(), pos.clone()));
                }
            }
        }

        Ok(())
    }
}

#[wasm_bindgen]
pub fn cmc_init() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    console_error_panic_hook::set_once();
    trace!("Info:\n Git version: {}", GIT_VERSION);
}



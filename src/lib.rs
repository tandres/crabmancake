use crate::scene::Scene;
use crate::bus::{Sender, Receiver};
use crate::bus_manager::*;
use log::trace;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::Document;
use std::rc::Rc;
use error::CmcError;

const GIT_VERSION: &str = git_version::git_version!();

mod control_panel;
mod render_panel;
mod bus;
mod bus_manager;
mod key_state;
mod entity;
mod error;
mod render;
mod scene;
mod shape;
mod assets;
mod light;
mod uid;

#[wasm_bindgen]
pub struct CmcClient {
    ui_receiver: Receiver<UiMsg>,
    render_sender: Sender<RenderMsg>,
    last_time: f64,
}

#[wasm_bindgen]
impl CmcClient {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<CmcClient, JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let location = window.location();
        let document: Document = window.document().expect("should have a document on window");
        let bus_manager = Rc::new(BusManager::new(0));
        let canvas_side = document.get_element_by_id("canvasSide").ok_or(CmcError::missing_val("canvasSide"))?;

        let render_props = render_panel::RenderPanelProps {
            panel: canvas_side.clone(),
            bus_manager: bus_manager.clone(),
            scene: Scene::new([-3., 2., 3.], 640., 480.),
        };
        render_panel::RenderPanelModel::mount(&canvas_side, render_props);
        let panel = document.get_element_by_id("controlPanel").ok_or(CmcError::missing_val("controlPanel"))?;
        control_panel::ControlPanelModel::mount(&panel, control_panel::ControlPanelProps { bus_manager: bus_manager.clone()});
        let render_sender = bus_manager.render.new_sender();

        let models = assets::load_models(location.origin()?, &window).await?;
        for model in models {
            render_sender.send(RenderMsg::NewModel(model));
        }

        let client = CmcClient {
            last_time: js_sys::Date::now(),
            ui_receiver: bus_manager.ui.new_receiver(),
            render_sender,
        };
        Ok(client)
    }

    pub fn update(&mut self, elapsed_time: f32) -> Result<(), JsValue> {
        let _delta_t = elapsed_time as f64 - self.last_time;
        self.last_time = elapsed_time as f64;

        let ui_events = self.ui_receiver.read();
        for event in ui_events {
            match event.as_ref() {
                UiMsg::NewObject(uid) => {
                    self.render_sender.send(RenderMsg::NewObject(uid.clone(), "Cube_glb".to_string()));
                },
                UiMsg::SetTarget(uid) => {
                    self.render_sender.send(RenderMsg::SetTarget(uid.clone()))
                },
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



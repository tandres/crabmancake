use crate::{scene::Scene, shape::Shape, light::{Attenuator, Light}};
use crate::bus::{Sender, Receiver};
use crate::bus_manager::*;
use log::{trace, debug};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, Event, EventTarget, HtmlCanvasElement, WebGlRenderingContext as WebGL};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
use key_state::KeyState;
use error::CmcError;
use rand::prelude::*;

const GIT_VERSION: &str = git_version::git_version!();
const RUST_CANVAS: &str = "rustCanvas";

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
    callbacks: HashMap<String, Rc<Closure<dyn FnMut(Event)>>>,
    canvas_side: Element,
    document: Rc<Document>,
    key_state: Arc<RwLock<KeyState>>,
    bus_manager: Rc<BusManager>,
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
        let document = Rc::new(document);
        control_panel::ControlPanelModel::mount(&panel, control_panel::ControlPanelProps { bus_manager: bus_manager.clone()});
        let render_sender = bus_manager.render.new_sender();

        let models = assets::load_models(location.origin()?, &window).await?;
        for model in models {
            render_sender.send(RenderMsg::NewModel(model));
        }

        let mut client = CmcClient {
            callbacks: HashMap::new(),
            canvas_side,
            document,
            key_state: Arc::new(RwLock::new(KeyState::new())),
            last_time: js_sys::Date::now(),
            ui_receiver: bus_manager.ui.new_receiver(),
            bus_manager,
            render_sender,
        };
        Ok(client)
    }

    pub fn update(&mut self, elapsed_time: f32) -> Result<(), JsValue> {

        let delta_t = elapsed_time as f64 - self.last_time;
        self.last_time = elapsed_time as f64;

        let ui_events = self.ui_receiver.read();
        for event in ui_events {
            match event.as_ref() {
                UiMsg::NewObject(uid) => {
                    self.render_sender.send(RenderMsg::NewObject(uid.clone(), "Cube_glb".to_string()));
                },
            }
        }

        // let key_state = self.key_state.read().unwrap().clone();
        // {
        //     let mut key_state = self.key_state.write().unwrap();
        //     key_state.clear();
        // }
        // {
        //     let mut scene = self.scene.write().unwrap();
        //     scene.update_aspect(self.canvas.width() as f32, self.canvas.height() as f32);
        //     scene.update_from_key_state(&key_state);
        // }

        // for (_id, shape) in self.shapes.iter_mut() {
        //     crate::entity::update(&mut shape.entity, delta_t as f32);
        // }
        Ok(())
    }

    pub fn render(&self) {
        // let scene = {
        //     self.scene.read().unwrap().clone()
        // };

        // for (_id, shape) in self.shapes.iter() {
        //     shape.render(&self.web_gl, &scene, &self.lights)
        // }
    }

    // fn lookup_callback(&self, event: &str) -> Option<Rc<Closure<dyn FnMut(Event)>>> {
    //     self.callbacks.get(&event.to_string()).map(|i| i.clone())
    // }

    // fn add_callback(&mut self, event: &str, callback: Box<dyn FnMut(Event)>) -> Result<Rc<Closure<dyn FnMut(Event)>>, JsValue> {
    //     let callback = Rc::new(Closure::wrap(callback));
    //     self.callbacks.insert(event.to_string(), callback);
    //     // log::debug!("Total callbacks: {}", self.callbacks.len());
    //     Ok(self.lookup_callback(event)
    //         .ok_or(CmcError::missing_val(format!("Couldn't retrieve {}", event)))?)
    // }

}

#[wasm_bindgen]
pub fn cmc_init() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    console_error_panic_hook::set_once();
    trace!("Info:\n Git version: {}", GIT_VERSION);
}

fn attach_handler<E>(element: &E, event_str: &str, handler: Rc<Closure<dyn FnMut(Event)>>) -> Result<(), JsValue>
where
    E: AsRef<EventTarget>,
{
    element.as_ref().add_event_listener_with_callback(event_str, handler.as_ref().as_ref().unchecked_ref())?;
    Ok(())
}

fn detach_handler<E>(element: &E, event_str: &str, handler: Rc<Closure<dyn FnMut(Event)>>) -> Result<(), JsValue>
where
    E: AsRef<EventTarget>,
{
    element.as_ref().remove_event_listener_with_callback(event_str, handler.as_ref().as_ref().unchecked_ref())?;
    Ok(())
}

// fn attach_pointerlock_handler(client: &mut CmcClient) -> Result<(), JsValue> {
//     let mousemove_event = "mousemove";
//     let scene_clone = client.scene.clone();
//     let mousemove_handler = move |event: Event| {
//         let event = event.dyn_into::<web_sys::MouseEvent>();
//         if let Ok(event) = event {
//             let x = -event.movement_x() as f32;
//             let y = -event.movement_y() as f32;
//             {
//                 let mut scene = scene_clone.write().unwrap();
//                 scene.mouse_rotate([x, y, 0.]);
//             }
//         } else {
//             log::warn!("Failed to convert event into mouseevent");
//         }
//     };
//     let mousemove_callback = client.add_callback(mousemove_event, Box::new(mousemove_handler))?;

//     let document = client.document.clone();
//     let keydown_event = "keydown";
//     let key_state_clone = client.key_state.clone();
//     let keydown_handler = move | event: Event| {
//         let event = event.dyn_into::<web_sys::KeyboardEvent>();
//         if let Ok(event) = event {
//             log::info!("Keydown event: {}", event.code());
//             key_state_clone.write().unwrap().set_key(event.code());
//         } else {
//             log::warn!("Failed to convert event into keyboardevent");
//         }
//     };
//     let keydown_callback = client.add_callback(keydown_event, Box::new(keydown_handler))?;

//     let document_clone = client.document.clone();
//     let pointerlockchange_handler = move |_event: Event| {
//         let element = document_clone.pointer_lock_element();
//         log::debug!("pointerlockchange");
//         let result = if element.is_some() && element.unwrap().id().as_str() == RUST_CANVAS {
//             log::debug!("Attaching mousemove handler");
//             vec![
//                 attach_handler(document_clone.as_ref(), mousemove_event, mousemove_callback.clone()),
//                 attach_handler(document_clone.as_ref(), keydown_event, keydown_callback.clone()),
//             ]
//         } else {
//             log::debug!("Detaching mousemove handler");
//             vec![
//                 detach_handler(document_clone.as_ref(), mousemove_event, mousemove_callback.clone()),
//                 detach_handler(document_clone.as_ref(), keydown_event, keydown_callback.clone()),
//             ]
//         };
//         if let Err(e) = result.into_iter().collect::<Result<Vec<()>, JsValue>>() {
//             log::error!("Attach/Detach failed: {:?}", e);
//         }
//     };
//     let event = "pointerlockchange";
//     let pointerlockchange_callback = client.add_callback(event, Box::new(pointerlockchange_handler))?;
//     attach_handler(document.as_ref(), event, pointerlockchange_callback.clone())?;
//     attach_handler(document.as_ref(), "mozpointerlockchange", pointerlockchange_callback)?;

//     let pointerlockerror_handler = move |_: Event| {
//         log::error!("Pointerlock error!");
//     };
//     let pointerlockerror_event = "pointerlockerror";
//     let pointerlockerror_callback = client.add_callback(pointerlockerror_event, Box::new(pointerlockerror_handler))?;
//     attach_handler(document.as_ref(), pointerlockerror_event, pointerlockerror_callback.clone())?;
//     attach_handler(document.as_ref(), "mozpointerlockerror", pointerlockerror_callback)?;

//     Ok(())
// }

// fn attach_mouse_onclick_handler(client: &mut CmcClient) -> Result<(), JsValue> {
//     let event = "click";
//     let canvas_clone = client.canvas.clone();
//     let document_clone = client.document.clone();
//     let handler = move |_event: Event| {
//         let element = document_clone.pointer_lock_element();
//         if element.is_none() || element.unwrap().id().as_str() != RUST_CANVAS {
//             canvas_clone.request_pointer_lock();
//         };
//     };

//     let handler = client.add_callback(event, Box::new(handler))?;
//     attach_handler(client.canvas.as_ref(), event, handler)?;

//     Ok(())
// }




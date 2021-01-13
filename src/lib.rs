use crate::{scene::Scene, shape::Shape, render::RenderCache, light::{Attenuator, Light}};
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

const GIT_VERSION: &str = git_version::git_version!();
const RUST_CANVAS: &str = "rustCanvas";

mod control_panel;
mod network;
mod key_state;
mod entity;
mod error;
mod render;
mod scene;
mod shape;
mod assets;
mod light;

#[wasm_bindgen]
pub struct CmcClient {
    web_gl: WebGL,
    #[allow(dead_code)]
    rendercache: RenderCache,
    shapes: HashMap<String, Shape>,
    lights: Vec<Light>,
    callbacks: HashMap<String, Rc<Closure<dyn FnMut(Event)>>>,
    canvas_side: Element,
    document: Rc<Document>,
    canvas: Rc<HtmlCanvasElement>,
    scene: Arc<RwLock<Scene>>,
    key_state: Arc<RwLock<KeyState>>,
    last_time: f64,
}

#[wasm_bindgen]
impl CmcClient {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<CmcClient, JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let location = window.location();
        let document: Document = window.document().expect("should have a document on window");
        let canvas_side = document.get_element_by_id("canvasSide").ok_or(CmcError::missing_val("canvasSide"))?;
        let panel = document.get_element_by_id("controlPanel").ok_or(CmcError::missing_val("controlPanel"))?;
        let models = assets::load_models(location.origin()?, &window).await?;
        let document = Rc::new(document);
        control_panel::ControlPanelModel::mount(&panel, control_panel::ControlPanelProps { suffix: "what".to_string()});

        let canvas: Rc<HtmlCanvasElement> = Rc::new(setup_canvas(&document)?);
        let gl = setup_gl_context(&canvas, true)?;
        let rendercache = render::build_rendercache(&gl, &models).expect("Failed to create rendercache");
        log::info!("Available shapes");
        for key in rendercache.shape_renderers.keys() {
            log::info!("{}", key);
        }
        let scene = Arc::new(RwLock::new(Scene::new([-3., 2., 3.], 640., 480.)));
        let lights = vec![
            Light::new_spot([0.,1.,0.], [0.,0.,0.], [1.,1.,1.], 90., 100., 10.0, Attenuator::new_7m()),
            Light::new_point([5.,0.,0.], [1., 1., 1.], 5.0, Attenuator::new_7m()),
            Light::new_point([-5.,0.,0.], [1.,1.,1.], 5.0, Attenuator::new_7m()),
        ];
        let mut client = CmcClient {
            web_gl: gl,
            rendercache,
            shapes: HashMap::new(),
            lights,
            callbacks: HashMap::new(),
            canvas_side,
            document,
            canvas,
            scene,
            key_state: Arc::new(RwLock::new(KeyState::new())),
            last_time: js_sys::Date::now(),
        };

        attach_mouse_onclick_handler(&mut client)?;
        attach_pointerlock_handler(&mut client)?;

        Ok(client)
    }

    pub fn update(&mut self, elapsed_time: f32) -> Result<(), JsValue> {
        let (new_width, new_height) = look_up_resolution(self.canvas_side.client_width(), self.canvas_side.client_height());
        if new_width != self.canvas.width() || new_height != self.canvas.height() {
            self.canvas.set_width(new_width);
            self.canvas.set_height(new_height);
            self.web_gl.viewport(0, 0, new_width as i32, new_height as i32);
        }

        let delta_t = elapsed_time as f64 - self.last_time;
        self.last_time = elapsed_time as f64;

        let key_state = self.key_state.read().unwrap().clone();
        {
            let mut key_state = self.key_state.write().unwrap();
            key_state.clear();
        }
        {
            let mut scene = self.scene.write().unwrap();
            scene.update_aspect(self.canvas.width() as f32, self.canvas.height() as f32);
            scene.update_from_key_state(&key_state);
        }

        for (_id, shape) in self.shapes.iter_mut() {
            crate::entity::update(&mut shape.entity, delta_t as f32);
        }
        Ok(())
    }

    pub fn render(&self) {
        self.web_gl.clear(WebGL::COLOR_BUFFER_BIT | WebGL::DEPTH_BUFFER_BIT);
        let scene = {
            self.scene.read().unwrap().clone()
        };

        for (_id, shape) in self.shapes.iter() {
            shape.render(&self.web_gl, &scene, &self.lights)
        }
    }

    fn lookup_callback(&self, event: &str) -> Option<Rc<Closure<dyn FnMut(Event)>>> {
        self.callbacks.get(&event.to_string()).map(|i| i.clone())
    }

    fn add_callback(&mut self, event: &str, callback: Box<dyn FnMut(Event)>) -> Result<Rc<Closure<dyn FnMut(Event)>>, JsValue> {
        let callback = Rc::new(Closure::wrap(callback));
        self.callbacks.insert(event.to_string(), callback);
        // log::debug!("Total callbacks: {}", self.callbacks.len());
        Ok(self.lookup_callback(event)
            .ok_or(CmcError::missing_val(format!("Couldn't retrieve {}", event)))?)
    }

}

#[wasm_bindgen]
pub fn cmc_init() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    console_error_panic_hook::set_once();
    trace!("Info:\n Git version: {}", GIT_VERSION);
}


fn look_up_resolution(avail_width: i32, avail_height: i32) -> (u32, u32) {
    let resolutions = [
        (320, 240),
        (640, 480),
        (1024, 768),
    ];
    let mut good_resolution = resolutions[0];
    for resolution in resolutions.iter() {
        if avail_width < resolution.0 as i32 || avail_height < resolution.1 as i32 {
            break;
        } else {
            good_resolution = resolution.clone();
        }
    }
    good_resolution
}

fn setup_canvas(document: &Rc<Document>) -> Result<HtmlCanvasElement, JsValue> {
    let canvas = document.get_element_by_id(RUST_CANVAS).ok_or(CmcError::missing_val(RUST_CANVAS))?;
    let canvas = canvas.dyn_into::<HtmlCanvasElement>()?;
    Ok(canvas)
}

fn setup_gl_context(canvas: &Rc<HtmlCanvasElement>, print_context_info: bool) -> Result<web_sys::WebGlRenderingContext, JsValue> {
    let context: web_sys::WebGlRenderingContext = canvas
        .get_context("webgl")?
        .ok_or(JsValue::from_str("Failed to get webgl context"))?
        .dyn_into()?;

    if print_context_info {
        debug!("Max Vertex Attributes: {}", WebGL::MAX_VERTEX_ATTRIBS);
        debug!("Max Vertex Uniform vectors: {}", WebGL::MAX_VERTEX_UNIFORM_VECTORS);
        debug!("Max Fragment Uniform vectors: {}", WebGL::MAX_FRAGMENT_UNIFORM_VECTORS);
        debug!("Max Texture Size: {}", WebGL::MAX_TEXTURE_SIZE);
    }

    context.enable(WebGL::DEPTH_TEST);
    context.enable(WebGL::BLEND);
    context.blend_func(WebGL::SRC_ALPHA, WebGL::ONE_MINUS_SRC_ALPHA);
    context.clear_color(0.5, 0.5, 0.5, 1.);
    context.clear_depth(1.);
    Ok(context)
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

fn attach_pointerlock_handler(client: &mut CmcClient) -> Result<(), JsValue> {
    let mousemove_event = "mousemove";
    let scene_clone = client.scene.clone();
    let mousemove_handler = move |event: Event| {
        let event = event.dyn_into::<web_sys::MouseEvent>();
        if let Ok(event) = event {
            let x = -event.movement_x() as f32;
            let y = -event.movement_y() as f32;
            {
                let mut scene = scene_clone.write().unwrap();
                scene.mouse_rotate([x, y, 0.]);
            }
        } else {
            log::warn!("Failed to convert event into mouseevent");
        }
    };
    let mousemove_callback = client.add_callback(mousemove_event, Box::new(mousemove_handler))?;

    let document = client.document.clone();
    let keydown_event = "keydown";
    let key_state_clone = client.key_state.clone();
    let keydown_handler = move | event: Event| {
        let event = event.dyn_into::<web_sys::KeyboardEvent>();
        if let Ok(event) = event {
            log::info!("Keydown event: {}", event.code());
            key_state_clone.write().unwrap().set_key(event.code());
        } else {
            log::warn!("Failed to convert event into keyboardevent");
        }
    };
    let keydown_callback = client.add_callback(keydown_event, Box::new(keydown_handler))?;

    let document_clone = client.document.clone();
    let pointerlockchange_handler = move |_event: Event| {
        let element = document_clone.pointer_lock_element();
        log::debug!("pointerlockchange");
        let result = if element.is_some() && element.unwrap().id().as_str() == RUST_CANVAS {
            log::debug!("Attaching mousemove handler");
            vec![
                attach_handler(document_clone.as_ref(), mousemove_event, mousemove_callback.clone()),
                attach_handler(document_clone.as_ref(), keydown_event, keydown_callback.clone()),
            ]
        } else {
            log::debug!("Detaching mousemove handler");
            vec![
                detach_handler(document_clone.as_ref(), mousemove_event, mousemove_callback.clone()),
                detach_handler(document_clone.as_ref(), keydown_event, keydown_callback.clone()),
            ]
        };
        if let Err(e) = result.into_iter().collect::<Result<Vec<()>, JsValue>>() {
            log::error!("Attach/Detach failed: {:?}", e);
        }
    };
    let event = "pointerlockchange";
    let pointerlockchange_callback = client.add_callback(event, Box::new(pointerlockchange_handler))?;
    attach_handler(document.as_ref(), event, pointerlockchange_callback.clone())?;
    attach_handler(document.as_ref(), "mozpointerlockchange", pointerlockchange_callback)?;

    let pointerlockerror_handler = move |_: Event| {
        log::error!("Pointerlock error!");
    };
    let pointerlockerror_event = "pointerlockerror";
    let pointerlockerror_callback = client.add_callback(pointerlockerror_event, Box::new(pointerlockerror_handler))?;
    attach_handler(document.as_ref(), pointerlockerror_event, pointerlockerror_callback.clone())?;
    attach_handler(document.as_ref(), "mozpointerlockerror", pointerlockerror_callback)?;

    Ok(())
}

fn attach_mouse_onclick_handler(client: &mut CmcClient) -> Result<(), JsValue> {
    let event = "click";
    let canvas_clone = client.canvas.clone();
    let document_clone = client.document.clone();
    let handler = move |_event: Event| {
        let element = document_clone.pointer_lock_element();
        if element.is_none() || element.unwrap().id().as_str() != RUST_CANVAS {
            canvas_clone.request_pointer_lock();
        };
    };

    let handler = client.add_callback(event, Box::new(handler))?;
    attach_handler(client.canvas.as_ref(), event, handler)?;

    Ok(())
}




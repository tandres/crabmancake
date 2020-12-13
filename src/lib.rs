use crate::{scene::Scene, entity::Entity, shape::Shape, error::CmcError, render::RenderCache, light::{Attenuator, Light}};
use log::{trace, debug};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, Event, EventTarget, HtmlCanvasElement, HtmlInputElement, WebGlRenderingContext as WebGL};
use js_sys::Function;
use nalgebra::Vector3;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
use key_state::KeyState;

const GIT_VERSION: &str = git_version::git_version!();
const RUST_CANVAS: &str = "rustCanvas";

mod key_state;
mod entity;
mod error;
mod render;
mod scene;
mod shape;
mod state;
mod assets;
mod light;

#[wasm_bindgen]
pub struct CmcClient {
    web_gl: WebGL,
    #[allow(dead_code)]
    rendercache: RenderCache,
    shapes: Vec<Shape>,
    lights: Vec<Light>,
    callbacks: HashMap<String, Rc<Closure<dyn FnMut(Event)>>>,
    document: Rc<Document>,
    canvas: Rc<HtmlCanvasElement>,
    scene: Arc<RwLock<Scene>>,
    key_state: Arc<RwLock<KeyState>>,
}

#[wasm_bindgen]
impl CmcClient {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<CmcClient, JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let location = window.location();
        let document: Document = window.document().expect("should have a document on window");
        let body = document.body().expect("No body!");

        let models = assets::load_models(location.origin()?, &window).await?;

        let (label, slider) = create_slider(&document, "X", 0.0..360.0, 0.0, |x| state::update_shape_rotation(0, x))?;
        body.append_child(&label)?;
        body.append_child(&slider)?;

        let (label, slider) = create_slider(&document, "Y", 0.0..360.0, 0.0, |x| state::update_shape_rotation(1, x))?;
        body.append_child(&label)?;
        body.append_child(&slider)?;

        let (label, slider) = create_slider(&document, "Z", 0.0..360.0, 0.0, |x| state::update_shape_rotation(2, x))?;
        body.append_child(&label)?;
        body.append_child(&slider)?;

        let (label, slider) = create_slider(&document, "Spot limit", 0.0..180.0, 90.0, |x| state::update_limit(x))?;
        body.append_child(&label)?;
        body.append_child(&slider)?;

        let (label, slider) = create_slider(&document, "X", -10.0..10.0, 0.0, |x| state::update_light_location(0, x))?;
        body.append_child(&label)?;
        body.append_child(&slider)?;

        let (label, slider) = create_slider(&document, "Y", -10.0..10.0, 2.0, |x| state::update_light_location(1, x))?;
        body.append_child(&label)?;
        body.append_child(&slider)?;

        let (label, slider) = create_slider(&document, "Z", -10.0..10.0, 0.0, |x| state::update_light_location(2, x))?;
        body.append_child(&label)?;
        body.append_child(&slider)?;
        let document = Rc::new(document);
        let canvas: Rc<HtmlCanvasElement> = Rc::new(setup_canvas(&document)?);
        let gl = setup_gl_context(&canvas, true)?;
        let rendercache = render::build_rendercache(&gl, &models).expect("Failed to create rendercache");
        log::info!("Available shapes");
        for key in rendercache.shape_renderers.keys() {
            log::info!("{}", key);
        }
        let mut shapes = Vec::new();
        let mut entity_locs = Vec::new();
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    entity_locs.push([i as f32 * 4., j as f32 * 4., k as f32 * 4.]);
                }
            }
        }
        for loc in entity_locs.iter() {
            let entity = Entity::new_at(Vector3::new(loc[0], loc[1], loc[2]));
            let cube_renderer = rendercache.get_shaperenderer("Cube_glb").expect("Failed to get renderer");
            shapes.push(Shape::new(cube_renderer, entity));
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
            shapes,
            lights,
            callbacks: HashMap::new(),
            document,
            canvas,
            scene,
            key_state: Arc::new(RwLock::new(KeyState::new())),
        };

        attach_mouse_onclick_handler(&mut client)?;
        attach_pointerlock_handler(&mut client)?;

        Ok(client)
    }

    pub fn update(&mut self, elapsed_time: f32, height: f32, width: f32) -> Result<(), JsValue> {
        let state = state::get_curr();
        self.lights[0].set_location(state.light_location);
        let delta_t = state::update(elapsed_time, height, width);
        let rotations = state::get_curr().rotations;
        let rotations = Vector3::new(
            rotations[0] as f32 * std::f32::consts::PI / 180.,
            rotations[1] as f32 * std::f32::consts::PI / 180.,
            rotations[2] as f32 * std::f32::consts::PI / 180.,
        );
        let key_state = self.key_state.read().unwrap().clone();
        {
            let mut key_state = self.key_state.write().unwrap();
            key_state.clear();
        }
        {
            let mut scene = self.scene.write().unwrap();
            scene.update_aspect(width, height);
            scene.update_from_key_state(&key_state);
        }

        for shape in self.shapes.iter_mut() {
            crate::entity::update(&mut shape.entity, delta_t);
            crate::entity::set_rotation(&mut shape.entity, rotations);
        }
        Ok(())
    }

    pub fn render(&self) {
        self.web_gl.clear(WebGL::COLOR_BUFFER_BIT | WebGL::DEPTH_BUFFER_BIT);
        let scene = {
            self.scene.read().unwrap().clone()
        };

        for shape in self.shapes.iter() {
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

fn create_slider<F>(document: &Document, label: &str, range: std::ops::Range<f32>, start: f32, mut func: F) -> Result<(Element, HtmlInputElement), JsValue>
where
    F: FnMut(f64) + 'static,
{

    let html_label = document.create_element("p")?;
    html_label.set_inner_html(label);
    let base = document.create_element("input")?;
    base.set_attribute("type", "range")?;
    base.set_attribute("min", &range.start.to_string())?;
    base.set_attribute("max", &range.end.to_string())?;
    base.set_attribute("value", &start.to_string())?;
    let html_input: HtmlInputElement = base.dyn_into::<HtmlInputElement>()?;
    let handler = move |event: web_sys::Event| {
        if let Some(target) = event.target() {
            if let Some(target_inner) = target.dyn_ref::<HtmlInputElement>() {
                let value = target_inner.value_as_number();
                func(value);
            }
        }
    };
    let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    html_input.add_event_listener_with_callback("input", &Function::from(handler.into_js_value()))?;
    Ok((html_label, html_input))
}


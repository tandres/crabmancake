use crate::{entity::Entity, shape::Shape, error::CmcError, render::SimpleRenderer};
use log::{info, trace};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlCanvasElement, WebGlRenderingContext as WebGL};
use std::rc::Rc;

const GIT_VERSION: &str = git_version::git_version!();

mod common;
mod entity;
mod error;
mod render;
mod shaders;
mod shape;
mod state;

#[wasm_bindgen]
pub struct CmcClient {
    web_gl: WebGL,
    shapes: Vec<Shape<SimpleRenderer>>,
}

#[wasm_bindgen]
impl CmcClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<CmcClient, JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let document: Document = window.document().expect("should have a document on window");
        let gl = setup_gl_context(&document)?;
        let renderer = Rc::new(SimpleRenderer::new(&gl)?);
        let mut entity = Entity::new_stationary();
        entity::set_rot_rate(&mut entity, nalgebra::Vector3::z());
        let shape = Shape::new(renderer.clone(), entity);
        let client = CmcClient {
            web_gl: gl,
            shapes: vec![shape],
        };
        Ok(client)
    }

    pub fn say_hello(&self) {
        info!("Hello from wasm-rust!");
    }

    pub fn update(&mut self, elapsed_time: f32, height: f32, width: f32) -> Result<(), JsValue> {
        let delta_t = state::update(elapsed_time, height, width);
        for shape in self.shapes.iter_mut() {
            crate::entity::update(&mut shape.entity, delta_t);
        }
        Ok(())
    }

    pub fn render(&self) {
        trace!("Render called");
        let state = state::get_curr();

        self.web_gl.clear(WebGL::COLOR_BUFFER_BIT | WebGL::DEPTH_BUFFER_BIT);
        for shape in self.shapes.iter() {
            shape.render(&self.web_gl, state.canvas_height, state.canvas_width)
        }
    }
}


#[wasm_bindgen]
pub fn cmc_init() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    console_error_panic_hook::set_once();
    trace!("Info:\n Git version: {}", GIT_VERSION);
}

fn setup_gl_context(doc: &Document) -> Result<web_sys::WebGlRenderingContext, JsValue> {
    let canvas = doc.get_element_by_id("rustCanvas").ok_or(CmcError::missing_val("rustCanvas"))?;
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;
    let context: web_sys::WebGlRenderingContext = canvas.get_context("webgl")?.ok_or(JsValue::from_str("Failed to get webgl context"))?.dyn_into()?;
    context.enable(WebGL::BLEND);
    context.blend_func(WebGL::SRC_ALPHA, WebGL::ONE_MINUS_SRC_ALPHA);
    context.clear_color(0., 0., 0., 1.);
    context.clear_depth(1.);
    Ok(context)
}



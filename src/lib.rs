use crate::error::{CmcError};
use log::trace;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlCanvasElement, WebGlRenderingContext as WebGL};

const GIT_VERSION: &str = git_version::git_version!();

mod error;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Trace).unwrap();
    console_error_panic_hook::set_once();

    trace!("Info: {}", GIT_VERSION);
    let window = web_sys::window().expect("no global `window` exists");
    let document: Document = window.document().expect("should have a document on window");
    let gl = setup_gl_context(&document)?;
    draw_something(&gl)
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

fn draw_something(gl: &web_sys::WebGlRenderingContext) -> Result<(), JsValue> {
    gl.clear(WebGL::COLOR_BUFFER_BIT | WebGL::DEPTH_BUFFER_BIT);
    Ok(())
}

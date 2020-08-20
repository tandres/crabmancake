use log::trace;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::{Document, CanvasRenderingContext2d};
const GIT_VERSION: &str = git_version::git_version!();

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Trace).unwrap();
    console_error_panic_hook::set_once();

    trace!("Build: {}", GIT_VERSION);
    let window = web_sys::window().expect("no global `window` exists");
    let document: Document = window.document().expect("should have a document on window");
    two_dimension_context(&document).expect("Failed to initialize 2d render context");
    Ok(())
}

fn two_dimension_context(doc: &Document) -> Result<(), JsValue> {
    let canvas = doc.get_element_by_id("rustCanvas").unwrap();
    trace!("Element: {:?}", canvas);
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    let context = canvas.get_context("2d")?;
    trace!("Context: {:?}", context);
    let render: CanvasRenderingContext2d = context
        .ok_or(JsValue::from_str("wah wah"))?
        .dyn_into()?;
    let green = JsValue::from_str("green");
    render.set_fill_style(&green);
    render.fill_rect(10.0, 10.0, 100.0, 100.0);
    Ok(())
}


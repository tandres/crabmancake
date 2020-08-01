use log::trace;
use wasm_bindgen::prelude::*;

const GIT_VERSION: &str = git_version::git_version!();

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue>{
    console_log::init_with_level(log::Level::Trace).unwrap();
    console_error_panic_hook::set_once();

    trace!("Build: {}", GIT_VERSION);
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let val = document.create_element("p")?;
    val.set_inner_html("Hello from Rust!");

    body.append_child(&val)?;

    Ok(())
}


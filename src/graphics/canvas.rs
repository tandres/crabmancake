use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{HtmlCanvasElement, WebGlRenderingContext as GL};
use crate::{bus::Sender, bus_manager::RenderMsg};

pub struct Canvas {
    _callback: Closure<dyn FnMut()>,
    gl: web_sys::WebGlRenderingContext,
}

impl Canvas {
    pub fn new(canvas_id: &str, container_id: &str, sender: Sender<RenderMsg>) -> Self {
        let window = web_sys::window()
            .expect("Should have a window");
        let document = window
            .document()
            .expect("Should have document");
        let canvas: HtmlCanvasElement = document
            .get_element_by_id(canvas_id)
            .expect("Failed to get rust canvas!")
            .dyn_into::<HtmlCanvasElement>()
            .expect("Failed to convert element into HtmlCanvasElement");
        let gl_context: web_sys::WebGlRenderingContext = canvas
            .get_context("webgl")
            .expect("Failed to get glRenderingContext")
            .expect("Gl context empty")
            .dyn_into()
            .expect("Failed to convert context to type");
        let canvas_container = document
            .get_element_by_id(container_id)
            .expect("Couldn't get canvas container!");
        let callback: Box<dyn FnMut()> = Box::new(move || {
            let width = canvas_container.client_width();
            let height = canvas_container.client_height();
            let (new_width, new_height) = look_up_resolution(width, height);
            if new_width != canvas.width() || new_height != canvas.height() {
                canvas.set_width(new_width);
                canvas.set_height(new_height);
                sender.send(RenderMsg::Resize(new_width, new_height));
            }
        });
        let callback = Closure::wrap(callback);
        window.set_onresize(Some(callback.as_ref().unchecked_ref()));
        setup_gl_context(&gl_context);
        Self {
            _callback: callback,
            gl: gl_context,
        }
    }

    pub fn get_gl(&self) -> &web_sys::WebGlRenderingContext {
        &self.gl
    }
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

fn setup_gl_context(gl: &GL) {
    gl.enable(GL::DEPTH_TEST);
    gl.enable(GL::BLEND);
    gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
    gl.clear_color(0.5, 0.5, 0.5, 1.);
    gl.clear_depth(1.);
}

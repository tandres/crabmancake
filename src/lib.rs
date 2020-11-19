use crate::{entity::Entity, shape::Shape, error::CmcError, render::{RenderCache, ShapeRenderer}};
use log::{info, trace};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlCanvasElement, HtmlInputElement, WebGlRenderingContext as WebGL};
use js_sys::Function;
use nalgebra::{Isometry3, Perspective3, Point3, Vector3};
use include_dir::{include_dir, Dir};

const GIT_VERSION: &str = git_version::git_version!();

mod entity;
mod error;
mod render;
mod shape;
mod state;

const MODEL_DIR: Dir = include_dir!("models/");

#[wasm_bindgen]
pub struct CmcClient {
    web_gl: WebGL,
    #[allow(dead_code)]
    rendercache: RenderCache,
    shapes: Vec<Shape<ShapeRenderer>>,
}

#[wasm_bindgen]
impl CmcClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<CmcClient, JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let document: Document = window.document().expect("should have a document on window");
        let body = document.body().expect("No body!");
        let (label, slider) = create_slider(&document, "X", 0.0..360.0, 0.0, |x| state::update_shape_rotation(0, x))?;
        body.append_child(&label)?;
        body.append_child(&slider)?;

        let (label, slider) = create_slider(&document, "Y", 0.0..360.0, 0.0, |x| state::update_shape_rotation(1, x))?;
        body.append_child(&label)?;
        body.append_child(&slider)?;

        let (label, slider) = create_slider(&document, "Z", 0.0..360.0, 0.0, |x| state::update_shape_rotation(2, x))?;
        body.append_child(&label)?;
        body.append_child(&slider)?;

        let gl = setup_gl_context(&document)?;
        let rendercache = render::build_rendercache(&gl, &MODEL_DIR).expect("Failed to create rendercache");
        let mut shapes = Vec::new();
        const SHAPE_BLOCK_CNT : usize = 1;
        for i in 0..SHAPE_BLOCK_CNT {
            for j in 0..SHAPE_BLOCK_CNT {
                let mut entity = Entity::new_at(Vector3::new(i as f32 * 5., 0., j as f32 * 5.));
                //entity::set_rot_rate(&mut entity, std::f32::consts::PI * Vector3::y() * 0.25 * (i + (j + 1)) as f32);
                //let cube_renderer = rendercache.get_shaperenderer("Suzanne").expect("Failed to get renderer");
                let cube_renderer = rendercache.get_shaperenderer("test_triangle").expect("Failed to get renderer");
                let shape = Shape::new(cube_renderer, entity);
                shapes.push(shape);
            }
        }
        let entity = Entity::new_at(Vector3::new(2.,1.,0.));
        let cube_renderer = rendercache.get_shaperenderer("Cube_glb").expect("Failed to get renderer");
        shapes.push(Shape::new(cube_renderer, entity));
        let client = CmcClient {
            web_gl: gl,
            rendercache,
            shapes,
        };
        Ok(client)
    }

    pub fn say_hello(&self) {
        info!("Hello from wasm-rust!");
    }

    pub fn update(&mut self, elapsed_time: f32, height: f32, width: f32) -> Result<(), JsValue> {
        let delta_t = state::update(elapsed_time, height, width);
        let rotations = state::get_curr().rotations;
        let rotations = Vector3::new(
            rotations[0] as f32 * std::f32::consts::PI / 180.,
            rotations[1] as f32 * std::f32::consts::PI / 180.,
            rotations[2] as f32 * std::f32::consts::PI / 180.,
        );
        for shape in self.shapes.iter_mut() {
            crate::entity::update(&mut shape.entity, delta_t);
            crate::entity::set_rotation(&mut shape.entity, rotations);
        }
        Ok(())
    }

    pub fn render(&self) {
        // trace!("Render called");
        let state = state::get_curr();

        self.web_gl.clear(WebGL::COLOR_BUFFER_BIT | WebGL::DEPTH_BUFFER_BIT);

        let aspect: f32 = state.canvas_width / state.canvas_height;
        let eye_rot = Isometry3::rotation(Vector3::new(state.rotation_x_axis, state.rotation_y_axis, 0.));
        pub const FIELD_OF_VIEW: f32 = 45. * std::f32::consts::PI / 180.; //in radians
        pub const Z_FAR: f32 = 1000.;
        pub const Z_NEAR: f32 = 1.0;
        let eye    = eye_rot * Point3::new(-3.0, 5.0, -3.0);

        let target = Point3::new(0.0, 0.0, 0.0);
        let view   = Isometry3::look_at_rh(&eye, &target, &Vector3::y());

        let projection = Perspective3::new(aspect, FIELD_OF_VIEW, Z_NEAR, Z_FAR);
        for shape in self.shapes.iter() {
            shape.render(&self.web_gl, &view, &projection)
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

    attach_mouse_down_handler(&canvas)?;
    attach_mouse_up_handler(&canvas)?;
    attach_mouse_move_handler(&canvas)?;

    context.enable(WebGL::DEPTH_TEST);
    //context.enable(WebGL::BLEND);
    //context.blend_func(WebGL::SRC_ALPHA, WebGL::ONE_MINUS_SRC_ALPHA);
    context.clear_color(0., 0., 0., 1.);
    context.clear_depth(1.);
    Ok(context)
}

fn attach_mouse_down_handler(canvas: &HtmlCanvasElement) -> Result<(), JsValue> {
    let handler = move |event: web_sys::MouseEvent| {
        state::update_mouse_down(event.client_x() as f32, event.client_y() as f32, true);
    };

    let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mousedown", handler.as_ref().unchecked_ref())?;
    handler.forget();

    Ok(())
}

fn attach_mouse_up_handler(canvas: &HtmlCanvasElement) -> Result<(), JsValue> {
    let handler = move |event: web_sys::MouseEvent| {
        state::update_mouse_down(event.client_x() as f32, event.client_y() as f32, false);
    };

    let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mouseup", handler.as_ref().unchecked_ref())?;
    handler.forget();

    Ok(())
}

fn attach_mouse_move_handler(canvas: &HtmlCanvasElement) -> Result<(), JsValue> {
    let handler = move |event: web_sys::MouseEvent| {
        state::update_mouse_position(event.client_x() as f32, event.client_y() as f32);
    };

    let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mousemove", handler.as_ref().unchecked_ref())?;
    handler.forget();

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

use wasm_bindgen::JsCast;
use yew::prelude::*;
use web_sys::{Element, HtmlCanvasElement, WebGlRenderingContext as WebGL};
use yew::services::{RenderService, Task};
use yew::{html, Component, ComponentLink, Html, NodeRef, ShouldRender};
use crate::shape::Shape;
use crate::render::RenderCache;
use crate::bus::Receiver;
use crate::bus_manager::{BusManager, RenderMsg};
use crate::scene::Scene;
use crate::light::{Attenuator, Light};
use std::rc::Rc;
use std::collections::HashMap;
use yew::services::resize::{ResizeService, ResizeTask, WindowDimensions};

pub struct RenderPanelModel {
    link: ComponentLink<Self>,
    web_gl: Option<WebGL>,
    canvas: Option<HtmlCanvasElement>,
    node_ref: NodeRef,
    render_loop: Option<Box<dyn Task>>,
    resize_task: Option<Box<ResizeTask>>,
    panel: Element,
    rendermsg_receiver: Receiver<RenderMsg>,
    scene: Scene,
    lights: Vec<Light>,
    rendercache: RenderCache,
    shapes: HashMap<String, Shape>,
}

pub enum Msg {
    Render(f64),
    Resize(WindowDimensions),
    MouseMove(MouseEvent),
    MouseWheel(WheelEvent),
}


#[derive(Clone, Properties, PartialEq)]
pub struct RenderPanelProps {
    pub panel: Element,
    pub bus_manager: Rc<BusManager>,
    pub scene: Scene,
}

impl Component for RenderPanelModel {
    type Message = Msg;
    type Properties = RenderPanelProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let rendercache = RenderCache::new_empty().expect("Failed to create rendercache");
        RenderPanelModel {
            canvas: None,
            web_gl: None,
            link,
            node_ref: NodeRef::default(),
            render_loop: None,
            resize_task: None,
            panel: props.panel,
            rendermsg_receiver: props.bus_manager.render.new_receiver(),
            rendercache,
            scene: props.scene,
            shapes: HashMap::new(),
            lights: vec![
            Light::new_spot([0.,1.,0.], [0.,0.,0.], [1.,1.,1.], 90., 100., 10.0, Attenuator::new_7m()),
            Light::new_point([5.,0.,0.], [1., 1., 1.], 5.0, Attenuator::new_7m()),
            Light::new_point([-5.,0.,0.], [1.,1.,1.], 5.0, Attenuator::new_7m()),
            ],
        }
    }

    fn rendered(&mut self, first_render: bool) {
        log::info!("Rendered");
        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();

        let gl: WebGL = canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        if first_render {
            setup_gl_context(&gl, false);
        }

        self.canvas = Some(canvas);
        self.web_gl = Some(gl);

        if first_render {
            self.resize();
            let resize = self.link.callback(Msg::Resize);
            let handle = ResizeService::new().register(resize);

            self.resize_task = Some(Box::new(handle));
            // The callback to request animation frame is passed a time value which can be used for
            // rendering motion independent of the framerate which may vary.
            let render_frame = self.link.callback(Msg::Render);
            let handle = RenderService::request_animation_frame(render_frame);

            // A reference to the handle must be stored, otherwise it is dropped and the render won't
            // occur.
            self.render_loop = Some(Box::new(handle));
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Render(timestamp) => {
                let updates = self.rendermsg_receiver.read();
                for msg in updates {
                    match msg.as_ref() {
                        RenderMsg::NewModel(model) => {
                            log::info!("Received new model");
                            if let Some(ref web_gl) = self.web_gl {
                                let result = self.rendercache.add_model(web_gl, model.as_ref());
                                if result.is_err() {
                                    log::warn!("Failed to add model to render cache!");
                                }
                            }
                        },
                        RenderMsg::NewObject(uid, renderer_name) => {
                            log::info!("Recieved new object");
                            if let Some(renderer) = self.rendercache.get_renderer(renderer_name) {
                                let entity = crate::entity::Entity::new_at(nalgebra::Vector3::new(2.,2.,2.));
                                let object = crate::shape::Shape::new(renderer, entity);
                                self.scene.look_at([2., 2., 2.]);
                                self.shapes.insert(uid.into(), object);
                            } else {
                                log::warn!("Couldn't find the requested renderer: {}", renderer_name);
                            }

                        },
                    }
                }
                self.render_gl(timestamp);
            },
            Msg::Resize(_window_dimensions) => {
                log::info!("Resized");
                self.resize();
            },
            Msg::MouseMove(m) => {
                let (primary, aux, _) = mouse_buttons_pressed(m.buttons());
                if primary {
                    let rot_sensi = 0.01;
                    let x = -m.movement_x() as f32;
                    let y = -m.movement_y() as f32;
                    self.scene.rotate_2d_about_target(x * rot_sensi, y * rot_sensi);
                }
                if aux {
                    let strafe_sensi = 0.001;
                    let x = -m.movement_x() as f32;
                    let y = -m.movement_y() as f32;
                    self.scene.strafe(x * strafe_sensi, y * strafe_sensi);
                }
            },
            Msg::MouseWheel(m) => {
                let wheel_sensitivity = 0.05;
                let amount = if m.delta_y().is_sign_positive() {
                    1. + wheel_sensitivity
                } else {
                    1. - wheel_sensitivity
                };
                self.scene.scale(amount)
            }
        }
        false
    }

    fn view(&self) -> Html {
        html! {
            <canvas
                onmousemove=self.link.callback(Msg::MouseMove)
                onwheel=self.link.callback(Msg::MouseWheel)
                ref={self.node_ref.clone()} />
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}

fn mouse_buttons_pressed(input: u16) -> (bool, bool, bool) {
    // LMB most of the time
    const PRIMARY_MOUSE_BUTTON_MASK: u16 = 0x1;
    // Center
    const AUX_MOUSE_BUTTON_MASK: u16 = 0x4;
    // Right
    const SECONDARY_MOUSE_BUTTON_MASK: u16 = 0x2;
    (PRIMARY_MOUSE_BUTTON_MASK & input == PRIMARY_MOUSE_BUTTON_MASK,
     AUX_MOUSE_BUTTON_MASK & input == AUX_MOUSE_BUTTON_MASK,
     SECONDARY_MOUSE_BUTTON_MASK & input == SECONDARY_MOUSE_BUTTON_MASK)

}

fn setup_gl_context(context: &WebGL, print_context_info: bool) {
    if print_context_info {
        log::debug!("Max Vertex Attributes: {}", WebGL::MAX_VERTEX_ATTRIBS);
        log::debug!("Max Vertex Uniform vectors: {}", WebGL::MAX_VERTEX_UNIFORM_VECTORS);
        log::debug!("Max Fragment Uniform vectors: {}", WebGL::MAX_FRAGMENT_UNIFORM_VECTORS);
        log::debug!("Max Texture Size: {}", WebGL::MAX_TEXTURE_SIZE);
    }
    context.enable(WebGL::DEPTH_TEST);
    context.enable(WebGL::BLEND);
    context.blend_func(WebGL::SRC_ALPHA, WebGL::ONE_MINUS_SRC_ALPHA);
    context.clear_color(0.5, 0.5, 0.5, 1.);
    context.clear_depth(1.);
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

impl RenderPanelModel {
    pub fn mount(element: &Element, props: RenderPanelProps) -> ComponentLink<Self> {
        App::<RenderPanelModel>::new().mount_with_props(element.clone(), props)
    }

    fn render_gl(&mut self, _timestamp: f64) {
        let gl = self.web_gl.as_ref().expect("GL Context not initialized!");
        gl.clear(WebGL::COLOR_BUFFER_BIT | WebGL::DEPTH_BUFFER_BIT);

        let scene = {
            self.scene.clone()
        };
        if let Some(ref web_gl) = self.web_gl {
            for (_id, shape) in self.shapes.iter() {
                shape.render(web_gl, &scene, &self.lights)
            }
        }

        let render_frame = self.link.callback(Msg::Render);
        let handle = RenderService::request_animation_frame(render_frame);

        // A reference to the new handle must be retained for the next render to run.
        self.render_loop = Some(Box::new(handle));
    }

    fn resize(&mut self) {
        if let (Some(canvas), Some(gl)) = (self.canvas.as_ref(), self.web_gl.as_ref()) {
            let (new_width, new_height) = look_up_resolution(self.panel.client_width(), self.panel.client_height());
            if new_width != canvas.width() || new_height != canvas.height() {
                canvas.set_width(new_width);
                canvas.set_height(new_height);
                gl.viewport(0, 0, new_width as i32, new_height as i32);
            }
        }
    }
}

use asset_list::get_asset_list;
use gltf::{buffer::Source as BufSource, Gltf, image::Source as ImgSource};
use crate::{bus::Sender, bus_manager::{BusManager, RenderMsg}};
use image::DynamicImage;
use std::{collections::HashMap, path::Path, rc::Rc};
use web_sys::Element;
use yew::services::fetch::{FetchService, FetchTask, Request, Response, Uri};
use yew::{format::Nothing, prelude::*};

const ASSET_DIR: &str = "assets";

mod asset;
mod asset_cache;
mod asset_list;
mod config;

pub use asset::Asset;
pub use config::Config;
pub use asset_cache::AssetCache;

pub fn start_asset_fetch(asset_cache: &AssetCache) {
    let window = web_sys::window().unwrap();
    let http_root = window.location().origin().unwrap();
    for config in asset_list::get_asset_config_list() {
        let url_root = format!("{}/{}/", http_root, ASSET_DIR);
        asset_cache.request_asset(url_root, config.to_string());
    }
}

pub const MODEL_DIR: &str = "models";

pub struct Model {
    pub gltf: Gltf,
    pub buffers: Vec<Vec<u8>>,
    pub images: Vec<DynamicImage>,
}

pub struct AssetLoadOperation {
    #[allow(unused)]
    gltf_fetch_task: FetchTask,
    gltf: Option<Gltf>,
    buffer_fetch_tasks: Vec<FetchTask>,
    buffers: Vec<Vec<u8>>,
    image_fetch_tasks: Vec<FetchTask>,
    images: Vec<Vec<u8>>,
    pending: usize,
}

impl AssetLoadOperation {
    pub fn new(gltf_fetch_task: FetchTask) -> Self {
        AssetLoadOperation {
            gltf_fetch_task,
            gltf: None,
            buffer_fetch_tasks: Vec::new(),
            buffers: Vec::new(),
            image_fetch_tasks: Vec::new(),
            images: Vec::new(),
            pending: 1,
        }
    }

    fn check_send(&mut self, sender: &Sender<RenderMsg>) {
        if self.pending != 0 {
            return;
        }
        let mut images = Vec::new();
        images.append(&mut self.images);
        let images: Result<Vec<DynamicImage>, _> = images.into_iter().map(|i| image::load_from_memory(&i)).collect();

        if let (Ok(images), Some(gltf)) = (images, self.gltf.take()) {
            let mut buffers = Vec::new();
            buffers.append(&mut self.buffers);
            let model = Rc::new(Model {gltf, images, buffers});
            sender.send(RenderMsg::NewModel(model))
        }
    }
}

#[derive(Clone, Properties, PartialEq)]
pub struct AssetLoadProps {
    pub server_root: String,
    pub bus_manager: Rc<BusManager>,
}

pub enum Msg {
    ReceivedGltf(String, Result<Vec<u8>, anyhow::Error>),
    ReceivedBuffer(String, usize, Result<Vec<u8>, anyhow::Error>),
    ReceivedImage(String, usize, Result<Vec<u8>, anyhow::Error>),
}

pub struct AssetLoadModel {
    ops: HashMap<String, AssetLoadOperation>,
    link: ComponentLink<Self>,
    server_root: String,
    render_sender: Sender<RenderMsg>,
}

impl AssetLoadModel {
    pub fn mount_with_props(element: &Element, props: AssetLoadProps) -> ComponentLink<Self> {
        App::<AssetLoadModel>::new().mount_with_props(element.clone(), props)
    }

    fn view_fetching(&self) -> Html {
        let outstanding : usize = self.ops.iter().map(|(_, o)| o.pending).sum();
        if outstanding > 0 {
            html! { <p>{ format!("Fetching {} models", outstanding) }</p> }
        } else {
            html! { <p></p> }
        }
    }

    fn build_request(&self, item: &str, callback: Callback<Response<Result<Vec<u8>, anyhow::Error>>>) -> FetchTask {
        let uri = format!("{}/{}/{}", self.server_root, MODEL_DIR, item);
        let uri = uri.parse::<Uri>().unwrap();
        log::info!("Fetching resource: {}", uri);
        let request = Request::get(uri)
            .body(Nothing)
            .expect("Could not build request.");
        FetchService::fetch_binary(request, callback).expect("failed to start request")
    }

    fn load_buffers(&self, item: &str, gltf: &Gltf) -> Vec<FetchTask> {
        let mut output_buffers = Vec::new();
        for (index, buffer) in gltf.buffers().enumerate() {
            match buffer.source() {
                BufSource::Uri(uri) => {
                    let item_clone = item.to_string();
                    let callback = self.link.callback(move |response: Response<Result<Vec<u8>, _>>| {
                        Msg::ReceivedBuffer(item_clone.clone(), index, response.into_body())
                    });
                    output_buffers.push(self.build_request(uri, callback));
                }
                _ => log::warn!("Unhandled buffer type"),
            }
        }
        output_buffers
    }

    fn load_images(&self, item: &str, gltf: &Gltf) -> Vec<FetchTask> {
        let mut output_images = Vec::new();
        for (index, image) in gltf.images().enumerate() {
            match image.source() {
                ImgSource::Uri{ uri, mime_type: _ } => {
                    let item_clone = item.to_string();
                    let callback = self.link.callback(move |response: Response<Result<Vec<u8>, _>>| {
                        Msg::ReceivedImage(item_clone.clone(), index, response.into_body())
                    });
                    output_images.push(self.build_request(uri, callback));
                },
                _ => log::warn!("View image not handled!"),
            }
        }
        output_images
    }
}

impl Component for AssetLoadModel {
    type Message = Msg;
    type Properties = AssetLoadProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            ops: HashMap::new(),
            link,
            server_root: props.server_root,
            render_sender: props.bus_manager.render.new_sender(),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            for item in get_asset_list() {
                let extension = Path::new(item).extension().map(|s| s.to_str());
                if let Some(Some("gltf")) = extension {
                    let item_clone = item.to_string();
                    let callback = self.link.callback(move |response: Response<Result<Vec<u8>, _>>| {
                        Msg::ReceivedGltf(item_clone.clone(), response.into_body())
                    });

                    let fetch = self.build_request(item, callback);
                    let op = AssetLoadOperation::new(fetch);
                    self.ops.insert(item.to_string(), op);
                }
            }
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        use Msg::*;

        match msg {
            ReceivedGltf(item, response) => {
                let res = response.map(|d| Gltf::from_slice(&d[..]));
                match res {
                    Ok(Ok(gltf)) => {
                        log::info!("Successfully parsed gltf binary");
                        if self.ops.contains_key(&item) {
                            let image_fetches = self.load_images(&item, &gltf);
                            let buffer_fetches = self.load_buffers(&item, &gltf);
                            let op = self.ops.get_mut(&item).unwrap();
                            op.buffer_fetch_tasks = buffer_fetches;
                            op.pending += op.buffer_fetch_tasks.len();
                            op.image_fetch_tasks = image_fetches;
                            op.pending += op.image_fetch_tasks.len();
                            op.gltf = Some(gltf);
                            op.pending -= 1;
                            log::info!("{} Pending fetches", op.pending);
                            op.check_send(&self.render_sender);
                        }
                        true
                    },
                    Ok(Err(_)) => {
                        log::warn!("Failed to parse gltf binary!");
                        false
                    },
                    _ => {
                        log::warn!("Failed to fetch binary file!");
                        false
                    }
                }
            },
            ReceivedBuffer(item, index, response) => {
                if let (Some(op), Ok(buf)) = (self.ops.get_mut(&item), response) {
                    op.buffers.insert(index, buf);
                    op.pending -= 1;
                    op.check_send(&self.render_sender);
                }
                true
            },
            ReceivedImage(item, index, response) => {
                if let (Some(op), Ok(img)) = (self.ops.get_mut(&item), response) {
                    op.images.insert(index, img);
                    op.pending -= 1;
                    op.check_send(&self.render_sender);
                }
                true
            },
        }

    }

    fn view(&self) -> Html {
        html! {
            <>
                { self.view_fetching() }
            </>
        }
    }
}

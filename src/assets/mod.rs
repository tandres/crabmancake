use gltf::Gltf;
use image::DynamicImage;

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

pub struct Model {
    pub gltf: Gltf,
    pub buffers: Vec<Vec<u8>>,
    pub images: Vec<DynamicImage>,
}


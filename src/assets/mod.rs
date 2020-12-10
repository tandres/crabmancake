use crate::error::CmcResult;
use futures::{StreamExt, stream::FuturesUnordered};
use model::{build_fetcher, load_images, load_buffers};
use std::path::Path;
use asset_list::get_asset_list;
use web_sys::Window;
use gltf::Gltf;

mod asset_list;
mod model;

pub use model::Model;

pub const MODEL_DIR: &str = "models";

pub async fn load_models(server_root: String, window: &Window) -> CmcResult<Vec<Model>> {
    log::info!("Server root: {}", server_root);
    let fetchers = FuturesUnordered::new();
    let mut models = Vec::new();
    for item in get_asset_list() {
        let path = Path::new(item);
        let uri = format!("{}/{}/{}",server_root, MODEL_DIR, item);
        let extension = path.extension().unwrap().to_str();
        if let Some("gltf") = extension {
            fetchers.push(build_fetcher(uri.clone(), window));
        }
    }
    let fetch_results = fetchers.collect::<Vec<CmcResult<Vec<u8>>>>().await;
    for fetched in fetch_results {
        match fetched {
            Ok(buffer) => {
                let gltf = Gltf::from_slice(&buffer[..])?;
                let images = load_images(&gltf, server_root.as_str(), window).await?;
                let buffers = load_buffers(&gltf, server_root.as_str(), window).await?;
                models.push(Model {gltf, buffers, images});
            },
            Err(e) => {
                log::error!("Failed to fetch model: {}", e);
            },
        }
    }
    Ok(models)
}

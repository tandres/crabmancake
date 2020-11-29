use futures::{Future, stream::FuturesUnordered, StreamExt, TryFutureExt, TryStreamExt};
use crate::error::{CmcError, CmcResult};
use std::path::Path;
use asset_list::get_asset_list;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_streams::ReadableStream;
use web_sys::{Request, RequestInit, RequestMode, Response, Window};
use js_sys::Uint8Array;
use gltf::{mesh::Mesh, buffer::{Buffer, Source as BufSource}, image::Format, Gltf, image::Source as ImgSource};

mod asset_list;

const MODEL_DIR: &str = "models";

pub struct Model {
    gltf: Gltf,
    buffers: Vec<Vec<u8>>,
    images: Vec<Vec<u8>>,
}

async fn build_fetcher(uri: String, window: &Window) -> CmcResult<Vec<u8>> {
    log::info!("Fetching {}", uri);
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&uri, &opts)?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let response: Response = resp_value.dyn_into()?;

    let raw_body = response.body().ok_or(CmcError::missing_val("Response body"))?;

    let body = ReadableStream::from_raw(raw_body.dyn_into().map_err(|_| CmcError::conversion_failed("ReadableStream"))?);
    let stream = body
        .into_stream()
        .map_ok(|js| {
            log::info!("{:?}", js);
            Uint8Array::from(js).to_vec()
        })
        .map_err(|e| {
            CmcError::from(e)
        })
        .try_collect::<Vec<Vec<u8>>>()
        .map_ok(|v| {
            v.into_iter().flatten().collect::<Vec<u8>>()
        });
    let buffer: Vec<u8> = stream.await?;
    Ok(buffer)
}

pub async fn load_models(server_root: String, window: &Window) -> CmcResult<Vec<Model>> {
    log::info!("Server root: {}", server_root);
    let fetchers = FuturesUnordered::new();
    let mut models = Vec::new();
    for item in get_asset_list() {
        let path = Path::new(item);
        let uri = format!("{}/{}/{}",server_root, MODEL_DIR, item);
        log::info!("{}", uri);
        let extension = path.extension().unwrap().to_str();
        log::info!("Extension: {:?}", extension);
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

async fn load_buffers(gltf: &Gltf, server_root: &str, window: &Window) -> CmcResult<Vec<Vec<u8>>> {
    let mut output_buffers = Vec::new();
    for buffer in gltf.buffers() {
        log::info!("Loading binary buffer: {:?}", buffer.name());
        match buffer.source() {
            BufSource::Uri(uri) => {
                let uri = format!("{}/{}/{}",server_root, MODEL_DIR, uri);
                log::info!("Uri for image: {}", uri);
                if let Ok(buf) = build_fetcher(uri.clone(), window).await {
                    output_buffers.insert(buffer.index(), buf);
                } else {
                    log::warn!("Failed to fetch buffer: {}", uri);
                }
            },
            _ => log::warn!("Unhandled buffer type"),
        }
    }
    Ok(output_buffers)
}

async fn load_images(gltf: &Gltf, server_root: &str, window: &Window) -> CmcResult<Vec<Vec<u8>>> {
    let mut output_buffers = Vec::new();
    for image in gltf.images() {
        log::info!("Loading image: {:?}", image.name());
        match image.source() {
            ImgSource::Uri{ uri, mime_type: _ } => {
                let uri = format!("{}/{}/{}",server_root, MODEL_DIR, uri);
                log::info!("Uri for image: {}", uri);
                if let Ok(buf) = build_fetcher(uri.clone(), window).await {
                    output_buffers.insert(image.index(), buf);
                } else {
                    log::warn!("Failed to fetch image: {}", uri);
                }
            },
            _ => {
                log::warn!("View image not handled!");
            }
        }
    }
    Ok(output_buffers)
}


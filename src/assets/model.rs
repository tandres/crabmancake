use super::MODEL_DIR;
use futures::{TryFutureExt, TryStreamExt};
use crate::error::{CmcError, CmcResult};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_streams::ReadableStream;
use web_sys::{Request, RequestInit, RequestMode, Response, Window};
use js_sys::Uint8Array;
use gltf::{buffer::Source as BufSource, image::{Data as ImgData, Format}, Gltf, image::Source as ImgSource};


pub struct Model {
    pub gltf: Gltf,
    pub buffers: Vec<Vec<u8>>,
    pub images: Vec<gltf::image::Data>,
}

pub async fn build_fetcher(uri: String, window: &Window) -> CmcResult<Vec<u8>> {
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

pub async fn load_buffers(gltf: &Gltf, server_root: &str, window: &Window) -> CmcResult<Vec<Vec<u8>>> {
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

pub async fn load_images(gltf: &Gltf, server_root: &str, window: &Window) -> CmcResult<Vec<ImgData>> {
    let mut output_buffers = Vec::new();
    for image in gltf.images() {
        log::info!("Loading image: {:?}", image.name());
        match image.source() {
            ImgSource::Uri{ uri, mime_type: _ } => {
                let uri = format!("{}/{}/{}",server_root, MODEL_DIR, uri);
                log::info!("Uri for image: {}", uri);
                if let Ok(buf) = build_fetcher(uri.clone(), window).await {
                    let cursor = std::io::Cursor::new(buf);
                    let (info, mut reader) = png::Decoder::new(cursor).read_info()?;
                    let mut raw = vec![0; info.buffer_size()];
                    reader.next_frame(&mut raw)?;
                    log::info!("Image info: {:?}", info);
                    output_buffers.insert(image.index(), ImgData {
                        pixels: raw,
                        width: info.width,
                        height: info.height,
                        format: Format::R8G8B8A8,
                    });
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


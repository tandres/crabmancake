use crate::{bus::Sender, error::{CmcResult, CmcError}, bus_manager::{AssetMsg, BusManager}};
use std::{collections::HashMap, sync::{Arc, RwLock}, rc::Rc};
use super::{Asset, Config};
use reqwest::Client;
use futures::{FutureExt, future::join_all};
use wasm_bindgen_futures::spawn_local;

pub struct AssetCache {
    //TJA todos: Web Storage, IndexDB, Expiration
    cache: Arc<RwLock<HashMap<String, Asset>>>,
    bus_manager: Rc<BusManager>,
    http_client: Client,
}

impl AssetCache {
    pub fn new(bus_manager: Rc<BusManager>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            bus_manager,
            http_client: Client::new(),
        }
    }

    pub fn request_asset(&self, url_root: String, url_name: String) {
        //TODO: Complicate this by checking whether or not the file is present. Would require
        //rectifying the name in the config with the url.

        let sender = self.bus_manager.asset.new_sender();
        let cache = self.cache.clone();
        let client = self.http_client.clone();
        spawn_local(fetch_asset(url_root, url_name, client, sender, cache).map(|res| {
            match res {
                Ok(_) => (),
                Err(e) => log::error!("Failed to fetch asset: {}", e),
            }
        }));
    }

    pub fn get_asset_config(&self, name: &str) -> Option<Config> {
        let cache = self.cache.read().unwrap();
        cache.get(name).map(|a| a.get_config().clone())
    }

    pub fn get_asset_info(&self, name: &str) -> Option<String> {
        let cache = self.cache.read().unwrap();
        cache.get(name).map(|a| a.get_asset_info())
    }
}

fn build_url(url_root: &str, filename: &str) -> String {
    format!("{}/{}", url_root, filename)
}

async fn fetch_asset(url_root: String, filename: String, client: Client, sender: Sender<AssetMsg>, cache: Arc<RwLock<HashMap<String, Asset>>>) -> CmcResult<()> {
    let config_url = build_url(&url_root, &filename);
    let config = fetch_config(&config_url, &client).await;
    log::info!("New config: {:?}", config);
    let (prompt_files, deferrable_files) = match &config {
        Ok(config) => {
            let prompt_files = config.get_prompt_files().into_iter().map(|f| fetch_binary_file(&url_root, f, &client)).collect();
            let deferrable_files = config.get_deferrable_files().into_iter().map(|f| fetch_binary_file(&url_root, f, &client)).collect();
            (prompt_files, deferrable_files)
        },
        Err(e) => {
            log::error!("Failed to get config: {}", e);
            (Vec::new(), Vec::new())
        },
    };
    //This goofiness is due to the compiler giving me a type needs to be known at compile time
    //error when the match is replaced with a ? on the call to fetch config or this code is
    //moved into the OK() match arm.
    if let Ok(config) = config {
        let asset_name = config.name.clone();
        add_asset(&cache, &asset_name, Asset::new(&asset_name, &config));
        sender.send(AssetMsg::New(asset_name.clone(), config.clone()));
        let prompt_files = join_all(prompt_files)
            .await
            .into_iter()
            .collect::<Result<Vec<(String, Vec<u8>)>, CmcError>>()?;
        modify_asset(&cache, &asset_name, |asset| asset.add_files(prompt_files));
        sender.send(AssetMsg::Update(asset_name.clone()));
        let deferrable_files: Vec<(String, Vec<u8>)> = join_all(deferrable_files)
            .await
            .into_iter()
            .collect::<Result<Vec<(String, Vec<u8>)>, CmcError>>()?;
        modify_asset(&cache, &asset_name, |asset| asset.add_files(deferrable_files));
        modify_asset(&cache, &asset_name, |asset| asset.set_complete());
        sender.send(AssetMsg::Complete(asset_name));
    }
    Ok(())
}

async fn fetch_config(uri: &str, client: &Client) -> CmcResult<Config> {
    Ok(client.get(uri).send().await?.json().await?)
}

async fn fetch_binary_file(url_root: &str, filename: String, client: &Client) -> CmcResult<(String, Vec<u8>)> {
    let url = build_url(&url_root, &filename);
    let data = client
        .get(&url)
        .send()
        .await?
        .bytes()
        .await?
        .as_ref()
        .to_vec();
    Ok((filename, data))
}

fn add_asset<S: AsRef<str>>(cache: &Arc<RwLock<HashMap<String, Asset>>>, key: S, asset: Asset) {
    let mut cache = cache.write().unwrap();
    let old = cache.insert(key.as_ref().to_string(), asset);
    if let Some(_val) = old {
        log::warn!("Asset cache value overwritten!");
    }
}

fn modify_asset<F>(cache: &Arc<RwLock<HashMap<String, Asset>>>, key: &str, fun: F)
where
    F: FnOnce(&mut Asset) -> (),
{
    let mut cache = cache.write().unwrap();
    match cache.get_mut(key) {
        Some(mut val) => fun(&mut val),
        None => log::warn!("Failed to find key {}", key),
    }
}

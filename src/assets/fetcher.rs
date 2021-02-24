use crate::{bus::Sender, bus_manager::FetchMsg, error::CmcResult, assets::Config};
use reqwest::Client;
use std::sync::RwLock;

pub struct Fetcher {
    sender: Sender<FetchMsg>,
    client: Client,
    pending_assets: RwLock<Vec<String>>,
}

impl Fetcher {
    pub fn new(sender: Sender<FetchMsg>) -> Self {
        Fetcher{
            client: Client::new(),
            sender,
            pending_assets: RwLock::new(Vec::new()),
        }
    }

    #[allow(unused)]
    pub fn add_asset(&self, uri: String) {
        let list = vec![uri];
        self.add_asset_list(list);
    }

    pub fn add_asset_list(&self, mut list: Vec<String>) {
        let mut asset_list = self.pending_assets.write().unwrap();
        asset_list.append(&mut list);
    }

    pub async fn run(&self) {
        let new_assets = {
            let mut new_assets = Vec::new();
            let mut asset_list = self.pending_assets.write().unwrap();
            new_assets.append(&mut asset_list);
            new_assets
        };
        log::info!("Fetcher fetching {} assets", new_assets.len());
        let mut fetchables = Vec::new();
        for asset in new_assets {
            fetchables.push(self.get_asset(asset));
        }
        futures::future::join_all(fetchables).await;
    }

    pub async fn get_asset(&self, uri: String) {
        let config = self.get_config(uri).await;
        log::info!("New config: {:?}", config);
    }

    pub async fn get_config(&self, uri: String) -> CmcResult<Config> {
        Ok(self.client.get(&uri).send().await?.json().await?)
    }

    pub async fn get(&self, uri: String) -> CmcResult<Vec<u8>> {
        let data = self.client
            .get(&uri)
            .send()
            .await?
            .bytes()
            .await?
            .as_ref()
            .to_vec();
        Ok(data)
    }
}


use super::Config;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq)]
pub enum AssetState {
    Incomplete,
    Complete,
}

pub struct Asset {
    // Copy of the config name
    name: String,
    config: Config,
    state: AssetState,
    files: HashMap<String, Vec<u8>>,
}

impl Asset {
    pub fn new<S: AsRef<str>>(name: S, config: &Config) -> Self {
        Self {
            name: name.as_ref().to_string(),
            config: config.clone(),
            state: AssetState::Incomplete,
            files: HashMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_complete(&self) -> bool {
        self.state == AssetState::Complete
    }

    pub fn set_complete(&mut self) {
        self.state = AssetState::Complete;
    }

    pub fn add_files(&mut self, file_list: Vec<(String, Vec<u8>)>) {
        for (name, file) in file_list {
            let old = self.files.insert(name, file);
            if let Some(_old) = old {
                log::warn!("Overwrote file in asset");
            }
        }
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn get_asset_info(&self) -> String {
        let mut result = vec![format!("Name: {}\n Files ({}):", self.name, self.files.len())];
        for (key, file) in self.files.iter() {
            result.push(format!("{}: {} bytes", key, file.len()));
        }
        result.join("")
    }

    pub fn get_file<'a, S: AsRef<str>>(&'a self, name: S) -> Option<&'a Vec<u8>> {
        self.files.get(name.as_ref())
    }
}

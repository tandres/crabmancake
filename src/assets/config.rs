use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AssetType {
    #[serde(rename = "gltf_model")]
    GltfModel { gl_root: String, prompt_files: Vec<String>, deferrable_files: Vec<String> },
    #[serde(rename = "glb_model")]
    GlbModel { gl_root: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub asset_type: AssetType,
}

impl Config {
    //Used by the build system to populate resource directories
    #[allow(unused)]
    fn get_file_lists(&self) -> Vec<String> {
        let mut list = Vec::new();
        match &self.asset_type {
            AssetType::GltfModel { gl_root, prompt_files, deferrable_files } => {
                list.push(gl_root.clone());
                list.append(&mut prompt_files.clone());
                list.append(&mut deferrable_files.clone());
            },
            AssetType::GlbModel { gl_root } => {
                list.push(gl_root.clone());
            },
        }
        list

    }

    fn get_prompt_files(&self) -> Vec<String> {
        let mut list = Vec::new();
        match &self.asset_type {
            AssetType::GltfModel { gl_root, prompt_files, deferrable_files: _ } => {
                list.push(gl_root.clone());
                list.append(&mut prompt_files.clone());
            },
            AssetType::GlbModel { gl_root } => {
                list.push(gl_root.clone());
            },
        }
        list
    }

    fn get_deferable_files(&self) -> Vec<String> {
        let mut list = Vec::new();
        match &self.asset_type {
            AssetType::GltfModel { gl_root: _, prompt_files: _, deferrable_files } => {
                list.append(&mut deferrable_files.clone());
            },
            AssetType::GlbModel { gl_root: _ } => {
            },
        }
        list
    }
}

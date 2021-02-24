use std::{env, io::{self, BufReader, Write}, fs::{self, File}, path::{Path, PathBuf}};

include!("src/assets/config.rs");
const ASSET_CONFIG_LIST_NAME: &str = "asset_config_list.rs";
const ASSET_CONFIG_PATH: &str = "assets/configs/";
const ASSET_SOURCE_PATH: &str = "assets/exports/";
const ASSET_OUTPUT_PATH: &str = "assets/deploy/";

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let asset_path = create_directory(Path::new(ASSET_OUTPUT_PATH));
    let mut config_files = Vec::new();
    for item in fs::read_dir(Path::new(ASSET_CONFIG_PATH)).unwrap() {
        let path = item.unwrap().path();
        if !path.is_file() {
            continue
        }
        let config = verify_config_format(&path);
        let _ = verify_config_contents(&config);
        let mut file_list = verify_config_files_present(&config);
        config_files.push(path.file_name().unwrap().to_str().unwrap().to_string());
        file_list.push(path.clone());
        for file in file_list {
            let file_name = file.file_name().unwrap().to_str().unwrap();
            let dest = Path::new(&asset_path).join(file_name);
            fs::copy(&file, dest).expect("Failed to copy");
        }
    }
    let mut asset_list_file = create_file(&out_dir, ASSET_CONFIG_LIST_NAME);
    write!(asset_list_file, "const ASSET_LIST: &[&str] = &[\n").unwrap();
    for config_file_name in config_files {
        write!(asset_list_file, "\"{}\",\n", config_file_name).unwrap();
    }
    write!(asset_list_file, "];").unwrap();
}

fn create_file<DIR: AsRef<Path>, F: AsRef<Path>>(path: DIR, file_name: F) -> File {
    let dest_path = Path::new(path.as_ref()).join(file_name.as_ref());
    File::create(dest_path).unwrap()
}

fn create_directory<DIR: AsRef<Path>>(path: DIR) -> PathBuf {
    let path = PathBuf::from(path.as_ref());
    match fs::create_dir(&path) {
        Err(e) => {
            use io::ErrorKind::*;
            match e.kind() {
                AlreadyExists => path,
                _ => panic!("Failed to create path {:?} - Unexpected io error {}", path, e),
            }
        }
        _ => path,
    }
}

fn verify_config_format<S: AsRef<Path>>(path: S) -> Config {
    assert_eq!(path.as_ref().extension().map(|s| s.to_str()), Some(Some("json")));
    match File::open(path) {
        Ok(file) => {
            serde_json::from_reader(BufReader::new(file)).expect("Failed to parse file into Json!")
        },
        Err(e) => {
            panic!("Couldn't open config file! {}", e);
        },
    }
}

fn verify_config_contents(config: &Config) {
    match &config.asset_type {
        AssetType::GltfModel {gl_root, prompt_files: _, deferrable_files: _} => {
            assert!(gl_root.ends_with(".gltf"));
            verify_gltf_import(gl_root);
        },
        AssetType::GlbModel {gl_root} => {
            assert!(gl_root.ends_with(".glb"));
            verify_gltf_import(gl_root);
        },
    }
}

fn verify_gltf_import<P: AsRef<Path>>(path: P) {
    let gltf = Path::new(ASSET_SOURCE_PATH).join(path);
    let _gltf = gltf::import(gltf).expect("Failed to import gltf");
}

fn verify_config_files_present(config: &Config) -> Vec<PathBuf> {
    let list = config.get_file_lists();
    let mut full_paths = Vec::new();
    for item in list {
        let full_path = Path::new(ASSET_SOURCE_PATH).join(item);
        assert!(full_path.exists());
        full_paths.push(full_path);
    }
    full_paths
}





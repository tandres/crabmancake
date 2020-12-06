use std::env;
use std::io::Write;
use std::fs::{self, File};
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("asset_list.rs");
    let mut asset_list_file = File::create(dest_path).unwrap();

    let items = fs::read_dir(Path::new("models/")).unwrap();
    write!(asset_list_file, "const ASSET_LIST: &[&str] = &[\n").unwrap();
    for item in items {
        let item = item.unwrap();
        let path = item.path();
        if path.is_file() {
            let path = path.file_name().unwrap().to_str().unwrap();
            write!(asset_list_file, "\"{}\",\n", path).unwrap();
        }
    }
    write!(asset_list_file, "];").unwrap();
}

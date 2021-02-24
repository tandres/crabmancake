include!(concat!(env!("OUT_DIR"), "/asset_config_list.rs"));

const FAKE_ASSET_LIST: &[&str] = &["fake"];

pub fn get_asset_list() -> &'static[&'static str] {
    FAKE_ASSET_LIST
}

pub fn get_asset_config_list() -> &'static[&'static str] {
    ASSET_LIST
}

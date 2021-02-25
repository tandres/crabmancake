include!(concat!(env!("OUT_DIR"), "/asset_config_list.rs"));

pub fn get_asset_config_list() -> &'static[&'static str] {
    ASSET_LIST
}

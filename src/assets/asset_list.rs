
include!(concat!(env!("OUT_DIR"), "/asset_list.rs"));

pub fn get_asset_list() -> &'static[&'static str] {
    ASSET_LIST
}

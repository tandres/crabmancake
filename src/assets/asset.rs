use super::Config;

enum AssetState {
    Incomplete,
    AwaitingDeferred,
    Complete,
}

pub struct Asset {
    // Copy of the config name
    name: String,
    config: Config,
    state: AssetState,
}

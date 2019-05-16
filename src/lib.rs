pub mod client;
pub(crate) mod net;
pub(crate) mod utils;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub cache_dir: String,
    pub output_format: bool,
    pub verbosity: bool,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub centerdevice: client::CenterDeviceConfig,
}


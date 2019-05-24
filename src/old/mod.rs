mod net;
pub mod client;
mod utils;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub output_format: bool,
    pub verbosity: bool,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub centerdevice: client::CenterDeviceConfig,
}

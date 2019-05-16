pub mod old_client;
pub mod client;
pub(crate) mod net;
pub(crate) mod utils;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub output_format: bool,
    pub verbosity: bool,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub centerdevice: old_client::CenterDeviceConfig,
}


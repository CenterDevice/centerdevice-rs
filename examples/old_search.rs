use centerdevice::old_client::{self, CenterDeviceConfig};
use centerdevice::old_client::search::NamedSearches;
use std::env;

fn main() {
    let client_id = env::var_os("CENTERDEVICE_CLIENT_ID").expect("Environment variable 'CENTERDEVICE_CLIENT_ID' is not set.");
    let client_secret = env::var_os("CENTERDEVICE_CLIENT_SECRET").expect("Environment variable 'CENTERDEVICE_CLIENT_SECRET' is not set.");
    let access_token = env::var_os("CENTERDEVICE_ACCESS_TOKEN").expect("Environment variable 'CENTERDEVICE_ACCESS_TOKEN' is not set.");

    let config = CenterDeviceConfig {
        client_id: client_id.to_string_lossy().to_string(),
        client_secret: client_secret.to_string_lossy().to_string(),
        refresh_token: None,
        access_token: access_token.to_string_lossy().to_string(),
        api_base_url: "centerdevice.de".to_string()
    };

    let search_results= old_client::search::search_documents(
        &config.api_base_url,
        &config.access_token,
        None,
        None,
        Some("centerdevice"),
        NamedSearches::None
    ).expect("Search failed.");

    println!("{}", search_results);
}

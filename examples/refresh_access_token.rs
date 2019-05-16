use centerdevice::client::*;
use std::env;

fn main() {
    let client_id = env::var_os("CENTERDEVICE_CLIENT_ID").expect("Environment variable 'CENTERDEVICE_CLIENT_ID' is not set.");
    let client_secret = env::var_os("CENTERDEVICE_CLIENT_SECRET").expect("Environment variable 'CENTERDEVICE_CLIENT_SECRET' is not set.");
    let access_token = env::var_os("CENTERDEVICE_ACCESS_TOKEN").expect("Environment variable 'CENTERDEVICE_ACCESS_TOKEN' is not set.");
    let refresh_token = env::var_os("CENTERDEVICE_REFRESH_TOKEN").expect("Environment variable 'CENTERDEVICE_REFRESH_TOKEN' is not set.");

    let credentials = Credentials::new(
        client_id.to_string_lossy().to_string(),
        client_secret.to_string_lossy().to_string(),
        access_token.to_string_lossy().to_string(),
    );
    let refresh_token = refresh_token.to_string_lossy().to_string();

    let mut centerdevice = CenterDevice::new("centerdevice.de".to_string(), credentials);

    let token = centerdevice.refresh_access_token(&refresh_token)
        .expect("Search failed.");

    println!("Refreshed Access Token: '{:?}'", token);
}

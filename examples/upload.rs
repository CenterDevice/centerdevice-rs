use centerdevice::client::upload::Upload;
use centerdevice::{CenterDevice, Client, ClientCredentials, Token};

use mime_guess;
use std::env;
use std::path::Path;

fn main() {
    let client_id = env::var_os("CENTERDEVICE_CLIENT_ID")
        .expect("Environment variable 'CENTERDEVICE_CLIENT_ID' is not set.")
        .to_string_lossy()
        .to_string();
    let client_secret = env::var_os("CENTERDEVICE_CLIENT_SECRET")
        .expect("Environment variable 'CENTERDEVICE_CLIENT_SECRET' is not set.")
        .to_string_lossy()
        .to_string();
    let access_token = env::var_os("CENTERDEVICE_ACCESS_TOKEN")
        .expect("Environment variable 'CENTERDEVICE_ACCESS_TOKEN' is not set.")
        .to_string_lossy()
        .to_string();
    let refresh_token = env::var_os("CENTERDEVICE_REFRESH_TOKEN")
        .expect("Environment variable 'CENTERDEVICE_REFRESH_TOKEN' is not set.")
        .to_string_lossy()
        .to_string();

    let client_credentials = ClientCredentials::new(client_id, client_secret);
    let token = Token::new(access_token, refresh_token);
    let client = Client::with_token("centerdevice.de".to_string(), client_credentials, token);

    let file_path = "examples/upload.rs";
    let path = Path::new(file_path);
    let mime_type = mime_guess::get_mime_type(file_path);
    let upload = Upload::new(path, mime_type)
        .expect("Failed to create Upload for path")
        .title("Rust upload example")
        .author("Lukas Pustina")
        .tags(&["rust"]);

    let result = client.upload_file(upload).expect("Upload failed");

    println!("Result: {:#?}", result);
}

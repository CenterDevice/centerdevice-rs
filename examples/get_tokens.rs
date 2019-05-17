use centerdevice::client::errors::*;
use centerdevice::client::*;
use reqwest::IntoUrl;
use std::env;
use std::io;
use std::io::Write;

struct MyCodeProvider {}

impl CodeProvider for MyCodeProvider {
    fn get_code<T: IntoUrl>(&self, auth_url: T) -> Result<Code> {
        let auth_url = auth_url.into_url().expect("Failed to parse auth url");

        println!("Please authenticate at the following URL, wait for the redirect, enter the code into the terminal, and then press return ...");
        println!("\n\t{}\n", auth_url);
        print!("Authentication code: ");
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let code = input.trim();

        let code = Code::new(code.to_string());

        Ok(code)
    }
}

fn main() {
    let client_id =
        env::var_os("CENTERDEVICE_CLIENT_ID").expect("Environment variable 'CENTERDEVICE_CLIENT_ID' is not set.");
    let client_secret = env::var_os("CENTERDEVICE_CLIENT_SECRET")
        .expect("Environment variable 'CENTERDEVICE_CLIENT_SECRET' is not set.");
    let redirect_uri =
        env::var_os("CENTERDEVICE_REDIRECT_URI").expect("Environment variable 'CENTERDEVICE_REDIRECT_URI' is not set.");

    let client_credentials = ClientCredentials::new(
        client_id.to_string_lossy().to_string(),
        client_secret.to_string_lossy().to_string(),
    );
    let redirect_uri = redirect_uri.to_string_lossy().to_string();
    let code_provider = MyCodeProvider {};

    let client = Client::new("centerdevice.de".to_string(), client_credentials)
        .authorize_with_code_flow(&redirect_uri, &code_provider)
        .expect("API call failed.");

    let result = client.token();

    println!("Result: '{:?}'", result);
}

use std::{collections::HashMap, env::args};

use reqwest::{blocking::Client};
use super::cli;

fn init_headers() -> Vec<String> {
    vec![
        "User-Agent: SafeBuster/1.0".to_string(),
        "Accept: */*".to_string(),
    ]
}

fn prepare_headers(headers: &mut Option<Vec<String>>) {
    match headers {
        Some(header) => {
            println!("Existing headers: {:?}", header);
        }
        None => {
            println!("Headers are empty. Initializing...");
            *headers = Some(init_headers());
        }
    }
}

pub fn safe_buster(mut args: cli::Args) {
    let client: Client = Client::new();
    prepare_headers(&mut args.headers);
    println!("Hello");
}

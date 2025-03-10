use std::{collections::HashMap};

use reqwest::{blocking::Client, header::HeaderMap, header::HeaderName, header::HeaderValue};

use super::cli;

fn init_headers_with_defaults() -> HeaderMap {
    let mut hash = HeaderMap::new();

    let headers = vec![
        "Content-Type: application/json",
        "Accept: application/json",
        "Authorization: Bearer <token>",
        "User-Agent: SafeBuster/1.0",
        "Accept: */*",
    ];

    headers.iter().for_each(|header| {
        let mut parts = header.splitn(2, ':');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            let key = key.trim();
            let value = value.trim();

            if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(header_value) = HeaderValue::from_str(value) {
                    hash.insert(header_name, header_value);
                }
            }
        }
    });

    hash
}

fn init_headers_with_value(headers: Vec<String>) -> HeaderMap {
    let mut hash = HeaderMap::new();

    for header in headers {
        let mut parts = header.splitn(2, ':');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            let key = key.trim();
            let value = value.trim();

            // Convert key and value into HeaderName and HeaderValue
            if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(header_value) = HeaderValue::from_str(value) {
                    hash.insert(header_name, header_value);
                }
            }
        }
    }

    hash
}

fn prepare_headers(headers: Option<Vec<String>>) -> HeaderMap{
    let  headers_hash;
    if let Some(header) = headers{
        headers_hash = init_headers_with_value(header);
        return headers_hash;

    }else{
        headers_hash = init_headers_with_defaults();
        return headers_hash;
    }
}
fn craft_request(args: cli::Args, headers_hash: HeaderMap, client : Client){

    let res = client.get(args.url).headers(headers_hash).send().unwrap();
    println!("Response : {:#?}", res);


}
pub fn safe_buster( args: cli::Args) {
    let client: Client = Client::new();
    let headers = args.headers.clone();
    let headers_hash = prepare_headers(headers);
    println!("Headers {:#?}", headers_hash);
    craft_request(args, headers_hash, client);
}

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};




fn init_headers_with_defaults() -> HeaderMap {
    let mut hash = HeaderMap::new();

    let headers = [
        "Content-Type: application/json",
        "Accept: application/json",
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

pub fn prepare_headers(headers: Option<Vec<String>>) -> HeaderMap {
    headers.map(init_headers_with_value).unwrap_or_else(init_headers_with_defaults)
}

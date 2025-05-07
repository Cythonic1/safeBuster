use std::usize;

use reqwest::{header::HeaderValue, StatusCode};

pub mod cli;
pub mod filehandle;
pub mod safebuster;
pub mod shared;

const DEFAULT_STATUS_CODE: &[u16] = &[
    StatusCode::OK.as_u16(),
    StatusCode::CREATED.as_u16(),
    StatusCode::ACCEPTED.as_u16(),
    StatusCode::NON_AUTHORITATIVE_INFORMATION.as_u16(),
    StatusCode::NO_CONTENT.as_u16(),
    StatusCode::RESET_CONTENT.as_u16(),
    StatusCode::PARTIAL_CONTENT.as_u16(),
    StatusCode::MULTI_STATUS.as_u16(),
    StatusCode::ALREADY_REPORTED.as_u16(),
    StatusCode::IM_USED.as_u16(),
    StatusCode::MOVED_PERMANENTLY.as_u16(),
    StatusCode::FOUND.as_u16(),
    StatusCode::TEMPORARY_REDIRECT.as_u16(),
    StatusCode::UNAUTHORIZED.as_u16(),
    StatusCode::FORBIDDEN.as_u16(),
    StatusCode::METHOD_NOT_ALLOWED.as_u16(),
    StatusCode::BAD_REQUEST.as_u16(),
];

//(index, res of string);
#[derive(Clone, Debug)]
struct PartingFileInfo(String, String);
const FUZZ: &str = "FUZZ";

pub trait HeaderValeExt {
    fn to_string(&self) -> String;
}
impl HeaderValeExt for HeaderValue {
    fn to_string(&self) -> String {
        self.to_str().unwrap_or_default().to_string()
    }
}

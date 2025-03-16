use std::usize;

use reqwest::StatusCode;

pub mod cli;
pub mod safebuster;




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
    StatusCode::BAD_REQUEST.as_u16()
];

//(index, res of string);
struct PartingFileInfo(String, String);
const FUZZ :&str= "FUZZ";

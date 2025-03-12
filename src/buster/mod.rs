use std::usize;

use reqwest::StatusCode;

pub mod cli;
pub mod safebuster;




const DEFAULT_STATUS_CODE: &[StatusCode] = &[
    StatusCode::OK,
    StatusCode::CREATED,
    StatusCode::ACCEPTED,
    StatusCode::NON_AUTHORITATIVE_INFORMATION,
    StatusCode::NO_CONTENT,
    StatusCode::RESET_CONTENT,
    StatusCode::PARTIAL_CONTENT,
    StatusCode::MULTI_STATUS,
    StatusCode::ALREADY_REPORTED,
    StatusCode::IM_USED,
    StatusCode::MOVED_PERMANENTLY,
    StatusCode::FOUND,
    StatusCode::TEMPORARY_REDIRECT,
    StatusCode::UNAUTHORIZED,
    StatusCode::FORBIDDEN,
    StatusCode::METHOD_NOT_ALLOWED,
];

const FUZZ :&str= "FUZZ";

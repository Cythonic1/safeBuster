use clap::{Parser, ValueEnum};
use core::fmt;
use std::{path::PathBuf, str::FromStr, usize};

#[derive(Debug, Clone, ValueEnum)]
pub enum HTTPMethods {
    POST,
    GET,
}

impl fmt::Display for HTTPMethods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
impl FromStr for HTTPMethods {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HTTPMethods::GET),
            "POST" => Ok(HTTPMethods::POST),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Parser, Clone, Default)]
#[command(version, about, long_about)]
pub struct Args {
    #[arg(short, long)]
    pub url: String,

    #[arg(long = "H", help = "headers sperated by ','", value_delimiter = ',')]
    pub headers: Option<Vec<String>>,

    #[arg(short, long)]
    pub wordlist: PathBuf,

    #[arg(long = "X", value_enum)]
    pub method: Option<HTTPMethods>,

    #[arg(short, long, default_value_t = 5)]
    pub threads: usize,

    #[arg(
        short,
        long,
        help = "the body data of POST request empty by defaults",
        default_value = ""
    )]
    pub data: String,

    #[arg(long = "fs", help = "Filter By status code", value_delimiter = ',')]
    pub filter_status: Option<Vec<u16>>,

    #[arg(long = "fd", help = "Filter By body data ")]
    pub contain: Option<String>,

    #[arg(long = "fr", help = "Filter By reponse Length", value_delimiter = ',')]
    pub filter_reponse_len: Option<Vec<usize>>,

    #[arg(long = "cn", help = "concurent tasks", default_value_t = 100)]
    pub concurrent_tasks: usize,

    #[arg(short, long)]
    pub file: PathBuf,
}

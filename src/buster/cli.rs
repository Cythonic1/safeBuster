use std::{path::PathBuf, usize};
use clap::{Parser, ValueEnum};
use reqwest::StatusCode;

#[derive(Debug, Clone, ValueEnum)]
pub enum HTTPMethods {
    POST,
    GET,
}


#[derive(Debug, Parser, Clone)]
#[command(version, about,long_about)]
pub struct Args{
    #[arg(short, long)]
    pub url: String,

    #[arg(long = "H", help = "headers sperated by ','", value_delimiter = ',')]
    pub headers : Option<Vec<String>>,


    #[arg(short, long)]
    pub wordlist: PathBuf,

    #[arg(long = "X" , value_enum)]
    pub method: Option<HTTPMethods>,


    
    #[arg(short, long, default_value_t = 5)]
    pub threads : usize,

    #[arg(short, long, help = "the body data of POST request empty by defaults", default_value = "")]
    pub data : String,

    #[arg(long = "fs", help = "Filter By status code", value_delimiter = ',')]
    pub filter_status: Option<Vec<StatusCode>>,

    #[arg( long = "fd", help = "Filter By body data ")]
    pub contain: Option<String>,

    #[arg(long = "fr", help = "Filter By reponse Length", value_delimiter = ',')]
    pub filter_reponse_len: Option<Vec<usize>>,

    #[arg(long="cn", help="concurent tasks", default_value_t= 100)]
    pub concurrent_tasks : usize
}


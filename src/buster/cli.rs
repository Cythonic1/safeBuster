use std::{path::PathBuf, usize};

// Handle command lines;
use clap::Parser;



#[derive(Debug, Parser, Clone)]
#[command(version, about,long_about)]
pub struct Args{
    #[arg(short, long)]
    pub url: String,

    #[arg(long = "H", help = "headers sperated by ','", value_delimiter = ',')]
    pub headers : Option<Vec<String>>,


    #[arg(short, long)]
    pub wordlist: PathBuf,

    #[arg(long = "X" , default_value = "GET")]
    pub method: Option<String>,


    #[arg(short, long, help = "check the response for given value")]
    pub contain: Option<String>,
    
    #[arg(short, long, default_value_t = 5)]
    pub threads : usize,

    #[arg(short, long, help = "the body data of POST request empty by defaults")]
    pub data : Option<String>
}


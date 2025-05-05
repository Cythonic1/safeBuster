// This mainly to parse http files and make request according to them

use std::panic;

use super::cli::{self, Args, HTTPMethods};
use super::HeaderValeExt;
use super::PartingFileInfo;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue}
;
pub struct FileParsing {
    pub args: cli::Args,
    file_content: String,
}
impl FileParsing {
    pub fn new(args: Args) -> Self {
        FileParsing {
            args,
            file_content: "".to_string(),
        }
    }

    pub fn open_file(&mut self) {
        println!("{:?}", self.args.file);
        match std::fs::read_to_string(&self.args.file) {
            Ok(content) => self.file_content = content,
            Err(err) => {
                eprintln!("Error : {err}");
                std::process::exit(1);
            }
        };
    }

    fn extract_method_path(&mut self, line: String) {
        /* First index contain Method, Second
         * Second index contain the path and prams in case of GET
         * Third contain the HTTP version
         * Gonna Comment the line below because i change my mind i want to
         * Handle the file segmentation in the main execution this gonna be
         * function for extracting the ...
         */
        let path: Vec<String> = line.split(" ").into_iter().map(|s| s.to_string()).collect();

        /* Extract the method
         * 
         */
        let method_type = path[0].parse::<HTTPMethods>().ok();
        self.args.method = method_type;

        //match &self.args.method {
        //    Some(method) => match method {
        //        HTTPMethods::POST => self.extract_post_path(Some(path[1].clone())),
        //        HTTPMethods::GET => self.extract_get_path(Some(path[1].clone())),
        //    },
        //    None => {}
        //}
    }

    fn extract_post_path(&mut self, parts: Option<String>) {

        match parts {
            Some(path) => {
                self.args.url = format!("{}{}", self.args.url, path);
            }
            None => {
                self.args.url = format!("{}{}", self.args.url, "/");
            }
        }
    }

    fn read_until_char(input: &str, delimiter: &str) -> Option<super::PartingFileInfo> {
        println!("The len of the given strrring is {}", delimiter.len());
        let match_found_index = if let Some(index) = input.find(delimiter) {
            index
        } else {
            return None;
        };

        Some(super::PartingFileInfo(
            input[..match_found_index].to_string(),
            input[(match_found_index + delimiter.len())..].to_string(),
        ))
    }

    fn extract_headers(&mut self, file_remaining: String) {
        let headers_vec: Vec<String> = file_remaining
            .split("\r\n")
            .map(|s| s.to_string())
            .collect();

        self.args.headers = Some(headers_vec);
    }

    /* TODO: Add the functionlity to convert from enum to normal string inorder to constrcut the
    / request path
    */
    fn extract_hostname(&mut self, headers: HeaderMap) {
        let urlconst = headers.get("Host");
        match urlconst {
            Some(url) => {
                self.args.url = format!("http://{}", url.to_string());
            }
            None => {
                panic!("No host found");
            }
        }
    }


    fn init_headers_with_defaults() -> HeaderMap {
        let mut hash = HeaderMap::new();

        let headers = vec![
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
                //
                if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
                    if let Ok(header_value) = HeaderValue::from_str(value) {
                        hash.insert(header_name, header_value);
                    }
                }
            }
        }

        hash
    }
    fn prepare_headers(headers: Option<Vec<String>>) -> HeaderMap {
        let headers_hash;
        if let Some(header) = headers {
            headers_hash = FileParsing::init_headers_with_value(header);
            headers_hash
        } else {
            headers_hash = FileParsing::init_headers_with_defaults();
            headers_hash
        }
    }

    /*
     * Function to extract the path from the first line in the file for get method
     */
    fn extract_get_path(&mut self, parts: Option<String>) {
        match parts {
            Some(part) => {
                let path: Vec<String> =
                    part.split(" ").into_iter().map(|s| s.to_string()).collect();

                //NOTE: If '?' Does not exist no prameters then it gonna return the path[1] it self
                let isolated_path = path[1].clone();
                println!("{isolated_path:#?}");

                // NOTE: Removing match statement and use unwrap_or insted better option.
                self.args.url = format!("{}{}", self.args.url, isolated_path);
            }
            None => {
                panic!("Method not found");
            }
        }
    }

    fn handle_match_parsing(buf: Option<PartingFileInfo>) -> PartingFileInfo {
        if let Some(value) = buf {
            value
        } else {
            panic!("Error parsing");
        }
    }
    pub fn main_execution(&mut self) {
        let mut parsing = FileParsing::read_until_char(&self.file_content, "\r\n");
        // here we gonna do three things

        let first_line = FileParsing::handle_match_parsing(parsing);
        println!("{}", first_line.0);
        self.extract_method_path(first_line.0.clone());
       
        parsing = FileParsing::read_until_char(&first_line.1, "\r\n\r\n");
        let raw_headers = FileParsing::handle_match_parsing(parsing);
        self.extract_headers(raw_headers.0);
       
        // TODO: Change to take instade of clone
        let parsed_headers = FileParsing::prepare_headers(self.args.headers.clone());
        self.extract_hostname(parsed_headers.clone());
        self.extract_get_path(Some(first_line.0));
       
        println!("{:#?}", self.args);

    }
}

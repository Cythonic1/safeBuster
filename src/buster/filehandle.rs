// This mainly to parse http files and make request according to them

use reqwest::{header::HeaderMap, Method};

use super::cli::{self, HTTPMethods};
struct FileParsing {
    args: cli::Args,
}

impl FileParsing {
    fn new() -> Self {
        return FileParsing {
            args: cli::Args::default(),
        };
    }

    fn open_file(&self) -> String {
        match std::fs::read_to_string(&self.args.file) {
            Ok(content) => return content,
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
         */
        let path: Vec<String> = line.split(" ").into_iter().map(|s| s.to_string()).collect();

        /* Extract the method
         * and get the
         */
        let method_type = path[0].parse::<HTTPMethods>().ok();
        self.args.method = method_type;
        match &self.args.method {
            Some(method) => match method {
                HTTPMethods::POST => self.extract_post_path(Some(path[1].clone())),
                HTTPMethods::GET => self.extract_get_path(Some(path[1].clone())),
            },
            None => {}
        }
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
        let urlconst = headers.iter().find(|x| x.0 == "Host").map(|x| x.1.clone());
    }
    fn extract_get_path(&mut self, parts: Option<String>) {
        match parts {
            Some(part) => {
                let isolated_path = part.split("?").into_iter().next();
                match isolated_path {
                    Some(path) => {
                        self.args.url = format!("{}{}", self.args.url, path);
                    }
                    None => self.args.url = format!("{}{}", self.args.url, "/"),
                }
            }
            None => {
                panic!("Method not found");
            }
        }
    }
}

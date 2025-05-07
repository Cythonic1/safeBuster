// This mainly to parse http files and make request according to them


use super::cli::{self, Args, HTTPMethods};
use super::PartingFileInfo;
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
    fn extract_hostname(&mut self, headers: Option<Vec<String>>) {
        let host_header = headers.as_ref()
            .and_then(|h| h.iter().find(|header| header.contains("Host")))
            .ok_or("No Host header found").unwrap();

        let host_part = host_header.split(':')
            .nth(1)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or("Invalid Host header format").unwrap();

        self.args.url = format!("http://{}", host_part);
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
        self.extract_hostname(self.args.headers.clone());
        self.args = super::safebuster::search_fuzz(self.args.clone(), "Soemthing");

        // TODO: Change to take instade of clone
        // let _parsed_headers = super::shared::prepare_headers(self.args.headers.clone());
        self.extract_get_path(Some(first_line.0));

        self.args.data = raw_headers.1;
       
        println!("{:#?}", self.args);

    }
}


// This mainly to parse http files and make request according to them


use std::fs::{self};

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
        let path = match &self.args.file {
            Some(p) => p,
            None => panic!("No file was provided")
        };

        match fs::read_to_string(path){
            Ok(content) => self.file_content = content,
            Err(err) => {
                eprintln!("Error : {err}");
                std::process::exit(1);
            }
        };
    }

    fn extract_method(&mut self, line: &str) {
        /* First index contain Method, Second
         * Second index contain the path and prams in case of GET
         * Third contain the HTTP version
         * Gonna Comment the line below because i change my mind i want to
         * Handle the file segmentation in the main execution this gonna be
         * function for extracting the ...
         */
        let path: Vec<String> = line.split(" ").map(|s| s.to_string()).collect();

        /* Extract the method
         *
         */
        let method_type = path[0].parse::<HTTPMethods>().ok();
        self.args.method = method_type;
    }

    fn extract_post_path(&mut self, parts: Option<String>) {
        let base_url = self.args.url.as_ref().ok_or("Base URL is missing");
        match parts {
            Some(path) => {

                self.args.url = Some(format!("{}{}", base_url.unwrap(), path));
            }
            None => {
                self.args.url = Some(format!("{}{}", base_url.unwrap(), "/"));
            }
        }
    }

    fn read_until_char(input: &str, delimiter: &str) -> Option<super::PartingFileInfo> {
        let match_found_index = input.find(delimiter)?;

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


    fn extract_hostname(&self, headers: &Option<Vec<String>>) -> Option<String>{
        let host_header = match headers.as_ref().and_then(|h| h.iter().find(|header| header.contains("Host"))){
            Some(host) => host,
            None => panic!("Host not found")
        };
            

        let host_part = match host_header.split(':').nth(1).map(|s| s.trim()).filter(|s| !s.is_empty()){
            Some(host) => host,
            None => panic!("Host not found")
        };

        Some(format!("http://{}", host_part))
    }

    /*
     * Function to extract the path from the first line in the file for get method
     */
    fn extract_get_path(&mut self, parts: Option<String>) {
        match parts {
            Some(part) => {
                let path: Vec<String> = part.split(" ").map(|s| s.to_string()).collect();

                //NOTE: If '?' Does not exist no prameters then it gonna return the path[1] it self
                let isolated_path = &path[1];

                // NOTE: Removing match statement and use unwrap_or insted better option.
                let base_url = self.args.url.as_ref();
                match base_url {
                    Some(base) =>  {
                        Some(format!("{}{}", base, isolated_path));
                    },
                    None => panic!("No base URL found")
                }
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

    pub fn prepare_args_from_file(&mut self) {
        let mut parsing = FileParsing::read_until_char(&self.file_content, "\r\n");

        // Extracting HTTP method
        let first_line = FileParsing::handle_match_parsing(parsing);
        self.extract_method(&first_line.0);


        // Extracting headers from the file and put them in a vector
        parsing = FileParsing::read_until_char(&first_line.1, "\r\n\r\n");
        let raw_headers = FileParsing::handle_match_parsing(parsing);
        self.extract_headers(raw_headers.0);

        // URL
        self.args.url = self.extract_hostname(&self.args.headers);

        // Extracting GET prameters and URL path and append them to the URL
        self.extract_get_path(Some(first_line.0));

        // Pitting the Rest of the raw data into the data if it a post
        self.args.data = raw_headers.1;

    }
}

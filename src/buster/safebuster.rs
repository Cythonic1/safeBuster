use std::{fs, io::{self, BufRead}, sync::Arc};
use reqwest::{blocking::Client, header::{HeaderMap, HeaderName, HeaderValue}, StatusCode};
use super::cli;
use super::FUZZ;
use crossbeam_channel::{bounded, Sender, Receiver};


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
            if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(header_value) = HeaderValue::from_str(value) {
                    hash.insert(header_name, header_value);
                }
            }
        }
    }

    hash
}

// string cmd
// loop extract the fuzz place
// cost u=f"https{Fd}"
//
fn prepare_headers(headers: Option<Vec<String>>) -> HeaderMap{
    let  headers_hash;
    if let Some(header) = headers{
        headers_hash = init_headers_with_value(header);
        return headers_hash;

    }else{
        headers_hash = init_headers_with_defaults();
        return headers_hash;
    }
}

// take args then search for fuzz.
// replace the args after taking the word
// then pass the new args into the craft request to make request with new created 
// args that includes the word replaced.
pub fn search_fuzz(mut args: cli::Args, word: &str) -> cli::Args{
    let mut counter_occurences = 0;
    if args.url.contains(FUZZ){
        args.url = args.url.replace(FUZZ, word);
        counter_occurences += 1;
        // println!("URL after change is {:?}", args.url);
    }


    if let Some(data) = args.data.as_mut() {
        if data.contains(FUZZ) {
            *data = data.replace(FUZZ, word); // Dereference and assign back
            counter_occurences += 1;
        }
    }
    if let Some(headers) = args.headers.as_mut() {
        for header in headers.iter_mut() {
            if header.contains(FUZZ){
                counter_occurences += 1;
                *header = header.replace(FUZZ, word);
            }
        }

    }

    // println!("headers after change is {:#?}", args.headers);
    // println!("You have added {} FUZZ we will replace all of them", counter_occurences);
    
    args


}
fn craft_request(args: cli::Args, client : Client, word: String){

    let args_clone= search_fuzz(args.clone(), &word);
    let headers_hash = prepare_headers(args_clone.headers);
    // TODO : handle different HTTP methods
    // println!("Headers {:#?}", headers_hash);
    // if let Some(method) = args_clone.method {
    //     // for now assume it only post
    //     //
    //     let res = client.post(args_clone.url).headers(headers_hash).body(args_clone.data.unwrap()).timeout(std::time::Duration::from_secs(10)).send().expect("Something");
    //     let res_status = res.status();

    //     match res.text() {
    //         Ok(data) => {
    //             if let Some(expected_res) = args_clone.contain {

    //                 // println!("Word : {word}");
    //                 if data.contains(&expected_res){
    //                     println!("The word: {} has gives the expected results", word);
    //                     return;
    //                 }
    //             }else {
    //                 // if res_status == StatusCode::OK {
    //                 //     println!("The word: {} has gives the status code of 200", word);
    //                 //     return;
    //                 // }
    //                 return ;
    //             }
    //         }
    //         Err(err) => {
    //             eprintln!("Error getting response text: {}", err);
    //             return;
    //         }

    //     }
    //     return;

    // }
    let res = client.get(args_clone.url).headers(headers_hash).send().unwrap();
    let res_status = res.status();

    match res.text() {
        Ok(data) => {
            if let Some(expected_res) = args_clone.contain {
                if data.contains(&expected_res){
                    println!("The word: {} has gives the expected results", word);
                }
            }else {
                if res_status == StatusCode::OK {
                    println!("The word: {} has gives the status code of 200", word);
                }
            }
        }
        Err(err) => {
            eprintln!("Error getting response text: {}", err);
        }
        
    }

    // println!("Response : {:#?}", res);


}

pub fn safe_buster(args: cli::Args) {
    let client = Client::new();
    let file = fs::File::open(args.wordlist.clone()).expect("Failed to open wordlist");
    let reader = io::BufReader::new(file);

    let (sender, receiver): (Sender<String>, Receiver<String>) = bounded(args.threads * 5);

    let args = Arc::new(args);
    let client = Arc::new(client);

    let mut handles = vec![];

    for _ in 0..args.threads {
        let receiver = receiver.clone();
        let args = Arc::clone(&args);
        let client = Arc::clone(&client);

        let handle = std::thread::spawn(move || {
            for line in receiver {
                // Clone necessary data for this request
                let args_clone = (*args).clone();
                let client_clone = (*client).clone();
                craft_request(args_clone, client_clone, line);
            }
        });

        handles.push(handle);
    }

    // Read lines and send to workers with backpressure
    for line in reader.lines() {
        if let Ok(ok_line) = line {
            // This will block if the channel is full, preventing memory overload
            sender.send(ok_line).expect("Failed to send line to worker");
        }
    }

    // Close the channel by dropping the sender
    drop(sender);

    // Wait for all workers to finish
    for handle in handles {
        handle.join().unwrap();
    }
}


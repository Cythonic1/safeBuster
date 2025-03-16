use std::{ path::PathBuf, process::Command, sync::{atomic::{AtomicUsize, Ordering}, Arc,}};
use futures_util::io::BufReader;
use reqwest::{Client, header::{HeaderMap, HeaderName, HeaderValue}, StatusCode};
use tokio::{io::AsyncBufReadExt, task::JoinSet,time::{sleep, Duration}, sync::Semaphore };
use super::{cli::{self, HTTPMethods}, DEFAULT_STATUS_CODE};
use super::FUZZ;
use std::time::Instant;


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

    println!("{:?}", hash);
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


    if args.data.contains(FUZZ) {
            args.data = args.data.replace(FUZZ, word); // Dereference and assign back
            counter_occurences += 1;
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
        
    return args;


}
fn filter_response(status_code: u16, res_body: &str, res_len: usize, filters: cli::Args) -> bool{
    // println!("\r Status code is {status_code}");
    let status_match =  if let Some(status_vec) = filters.filter_status {
        status_vec.contains(&status_code)
    }else {
        DEFAULT_STATUS_CODE.contains(&status_code)
    };

    // Filter by response length
    let length_match = filters.filter_reponse_len
        .as_ref()
        .map_or(false, |lengths| lengths.contains(&res_len));

    // Filter by response body containing specific text
    let content_match = filters.contain
        .as_ref()
        .map_or(false, |contents| contents.contains(&res_body));

    status_match || length_match || content_match

}

fn read_until_char(input: &str, delimiter: &str) -> Option<super::PartingFileInfo>{
    println!("The len of the given strrring is {}", delimiter.len());
    let match_found_index = if let Some(index) =  input.find(delimiter){
        index
    }else{
        return None
    };
     
    Some(super::PartingFileInfo(input[..match_found_index].to_string(), input[(match_found_index+delimiter.len())..].to_string()))

}

// TODO: Continue Parsing The File.
pub fn parse_file(file:PathBuf, _args: super::cli::Args) -> Option<Vec<String>> {
    
    let file_content = match std::fs::read_to_string(file){
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error : {err}");
            return None;
        }
    };
    let url = read_until_char(&file_content, "\r\n");
    let data_after_url = if let Some(data) = url {
        data
    }else{
        return None;
    };

    let headers = read_until_char(&data_after_url.1, "\r\n\r\n");

    let headers_vec: Vec<String> = headers.unwrap().0.replace("\r\n", ",").split(",").map(|s| s.to_string()).collect();

    let parsed_heaers = prepare_headers(Some(headers_vec));

    let urlconst = parsed_heaers.iter()
        .find(|x| x.0 == "Host")
        .map(|x| x.1.clone());
    println!("Host is : {:?}", urlconst.unwrap());


    Some(vec!["Hellow".to_string()])

}

async fn craft_request(args: cli::Args, client : Arc<Client>, word: String){

    let args_clone = search_fuzz(args.clone(), &word);
    let headers_hash = prepare_headers(args_clone.headers.clone());

    // Start measuring the request duration
    let start_time = Instant::now();

    let res = match args_clone.method {
        Some(HTTPMethods::GET) => match client
            .get(args_clone.url.clone())
            .headers(headers_hash)
            .send()
            .await
        {

            Ok(body) => body,
            Err(_) => return,
        },
        Some(HTTPMethods::POST) => match client
            .post(args_clone.url.clone())
            .headers(headers_hash)
            .body(args_clone.data.clone())
            .send()
            .await
        {
            Ok(body) => body,
            Err(_) => return,
        },
        None => {
            match client
                .get(args_clone.url.clone())
                .headers(headers_hash)
                .send()
            .await{
                Ok(body) => body,
                Err(_) => return
            }

        }
    };

    // Measure the duration
    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis();

    let status_code = res.status();

    match res.text().await {
        Ok(body) => {
            if filter_response(status_code.into(), &body, body.len(), args_clone) {
                let size = body.len();
                let words = body.split_whitespace().count();
                let lines = body.lines().count();

                // Print formatted output
                println!(
                    "\r{:<24} [Status: {}, Size: {}, Words: {}, Lines: {}, Duration: {}ms]",
                    word,
                    status_code.as_u16(),
                    size,
                    words,
                    lines,
                    duration_ms
                );
            }
        }
        Err(err) => {
            eprintln!("\rError: {err}");
        }
    }

}

pub async fn safe_buster(args: cli::Args) -> tokio::io::Result<()>{

    const MAX_CONCURRENT_TASKS: usize = 100;

    let semaphore = Arc::new(Semaphore::new(args.concurrent_tasks));
    let client = Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(1)) // Apply timeout at client level
            .build()
            .expect("Failed to create HTTP client"),
    );
    let file = tokio::fs::File::open(args.wordlist.clone()).await.expect("Failed to open wordlist");
    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();
    let mut tasks = JoinSet::new();
    let counter = Arc::new(AtomicUsize::new(0)); // Atomic counter for tracking progress
    let counter_clone = Arc::clone(&counter);
    let progress_task = tokio::spawn(async move { 
        loop {
            sleep(Duration::from_secs(1)).await;
            // print!("\x1B[999;0H"); // Moves to row 999 (forces it to the last line)
            print!("\rWords processed so far: {}", counter_clone.load(Ordering::Relaxed));
            std::io::Write::flush(&mut std::io::stdout()).unwrap(); // Force output refresh
        }
    });
    while let Some(word) = lines.next_line().await?{
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let args = args.clone();
        let client = Arc::clone(&client);
        let counter = Arc::clone(&counter);

        counter.fetch_add(1, Ordering::Relaxed);
        tasks.spawn(async move {
            let _permit = permit; // Keeps the semaphore permit alive
            craft_request(args, client, word).await;

        });
    }
    while let Some(_) = tasks.join_next().await {}
    progress_task.abort();
    println!("\rTotal words processed: {}", counter.load(Ordering::Relaxed));
    Ok(())

}


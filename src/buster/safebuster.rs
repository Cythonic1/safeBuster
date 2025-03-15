use std::sync::{atomic::{AtomicUsize, Ordering}, Arc,};
use clap::error::Result;
use reqwest::{Client, header::{HeaderMap, HeaderName, HeaderValue}, StatusCode};
use tokio::{io::{self, AsyncBufReadExt}, task::JoinSet,time::{sleep, Duration}, sync::Semaphore };
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
fn filter_response(status_code: StatusCode, res_body: &str, res_len: usize, filters: cli::Args) -> bool{
    filters.filter_status.as_ref().map_or(DEFAULT_STATUS_CODE.contains(&status_code), |code| code.contains(&status_code)) ||
    filters.filter_reponse_len.as_ref().map_or(false, |len|len.contains(&res_len)) || 
    filters.contain.as_ref().map_or(false, |content| content.contains(res_body))

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
            println!("No HTTP method provided.");
            return;
        }
    };

    // Measure the duration
    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis();

    let status_code = res.status();

    match res.text().await {
        Ok(body) => {
            if filter_response(status_code, &body, body.len(), args_clone) {
                let size = body.len();
                let words = body.split_whitespace().count();
                let lines = body.lines().count();

                // Print formatted output
                println!(
                    "{:<24} [Status: {}, Size: {}, Words: {}, Lines: {}, Duration: {}ms]",
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
            eprintln!("Error: {err}");
        }
    }

}

pub async fn safe_buster(args: cli::Args) -> tokio::io::Result<()>{

    const MAX_CONCURRENT_TASKS: usize = 100;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));
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
            println!("Words processed so far: {}", counter_clone.load(Ordering::Relaxed));
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
    println!("Total words processed: {}", counter.load(Ordering::Relaxed));
    Ok(())

}


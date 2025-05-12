use super::FUZZ;
use super::{ cli::{self, HTTPMethods},DEFAULT_STATUS_CODE};
use reqwest::Client;
use std::time::Instant;
use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};

use tokio::{ io::AsyncBufReadExt, sync::Semaphore, task::JoinSet, time::{sleep, Duration}};



// TODO: Recator this file and move the share function into share file.


pub fn search_fuzz(mut args: cli::Args, word: &str) -> cli::Args {
    let mut counter_occurences = 0;
    if let Some(ref url) = args.url {
        if url.contains(FUZZ) {
            args.url = Some(url.replace(FUZZ, word));
            counter_occurences += 1;
        }
    }

    if args.data.contains(FUZZ) {
        args.data = args.data.replace(FUZZ, word); // Dereference and assign back
        counter_occurences += 1;
    }
    if let Some(headers) = args.headers.as_mut() {
        for header in headers.iter_mut() {
            if header.contains(FUZZ) {
                counter_occurences += 1;
                *header = header.replace(FUZZ, word);
            }
        }
   }

    if counter_occurences == 0 {
        panic!("no FUZZ found");
    }


    args
}

fn filter_response(status_code: u16, res_body: &str, res_len: usize, filters: cli::Args) -> bool {
    // println!("\r Status code is {status_code}");
    let status_match = if let Some(status_vec) = filters.filter_status {
        status_vec.contains(&status_code)
    } else {
        DEFAULT_STATUS_CODE.contains(&status_code)
    };

    // Filter by response length
    let length_match = filters.filter_reponse_len.as_ref().is_some_and(|leng| leng.contains(&res_len));



    // Filter by response body containing specific text
    let content_match = filters.contain.as_ref().is_some_and(|match_cont| match_cont.contains(res_body));
    status_match || length_match || content_match
}


async fn craft_request(args: cli::Args, client: Arc<Client>, word: String) {
    let args_clone = search_fuzz(args.clone(), &word);
    let headers_hash = super::shared::prepare_headers(args_clone.headers.clone());

    // Start measuring the request duration
    let start_time = Instant::now();

    let res = match args_clone.method {
       Some(HTTPMethods::GET) => match client
            .get(args_clone.url.clone().unwrap())
            .headers(headers_hash)
            .send()
            .await
        {
            Ok(body) => body,
            Err(_) => return,
        },
        Some(HTTPMethods::POST) => match client
            .post(args_clone.url.clone().unwrap())
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
                .get(args_clone.url.clone().unwrap())
                .headers(headers_hash)
                .send()
                .await
            {
                Ok(body) => body,
                Err(_) => return,
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

pub async fn safe_buster(args: cli::Args) -> tokio::io::Result<()> {
    const MAX_CONCURRENT_TASKS: usize = 100;
    println!("am here");

    let semaphore = Arc::new(Semaphore::new(args.concurrent_tasks));
    let client = Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(1)) // Apply timeout at client level
            .build()
            .expect("Failed to create HTTP client"),
    );
    let file = tokio::fs::File::open(args.wordlist.clone())
        .await
        .expect("Failed to open wordlist");
    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();
    let mut tasks = JoinSet::new();
    let counter = Arc::new(AtomicUsize::new(0)); // Atomic counter for tracking progress
    let counter_clone = Arc::clone(&counter);
  
    let progress_task = tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            // print!("\x1B[999;0H"); // Moves to row 999 (forces it to the last line)
            print!(
                "\rWords processed so far: {}",
                counter_clone.load(Ordering::Relaxed)
            );
            std::io::Write::flush(&mut std::io::stdout()).unwrap(); // Force output refresh
        }
    });
    while let Some(word) = lines.next_line().await? {
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
    while  (tasks.join_next().await).is_some(){

    }
    progress_task.abort();
    println!(
        "\rTotal words processed: {}",
        counter.load(Ordering::Relaxed)
    );
    Ok(())
}

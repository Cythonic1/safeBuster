use super::FUZZ;
use super::{ cli::{self, HTTPMethods},DEFAULT_STATUS_CODE};
use reqwest::Client;
use tokio::task::JoinHandle;
use std::env::args;
use std::path::PathBuf;
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
        eprintln!("no FUZZ found");
        std::process::exit(1);
    }


    args
}

fn filter_response(status_code: u16, res_body: &str, res_len: usize, words: u32, filters: &cli::Args) -> bool {
    let status_match = if let Some(status_vec) = &filters.filter_status {
        status_vec.contains(&status_code)
    } else {
        DEFAULT_STATUS_CODE.contains(&status_code)
    };

    // Filter by response length
    let length_match = filters.filter_reponse_len.as_ref().is_some_and(|leng| leng.contains(&res_len));



    // Filter by response body containing specific text
    let content_match = filters.contain.as_ref().is_some_and(|match_cont| match_cont.contains(res_body));

    let words_match = if let Some(num_words) = &filters.filter_words {
        num_words.contains(&words)
    }else {
        false
    };
    // println!("{status_match}, {length_match}, {content_match}");
    status_match || length_match || content_match || words_match
}


async fn craft_request(args: cli::Args, client: Arc<Client>, word: String) {
    let args_clone = search_fuzz(args, &word);
    let headers_hash = super::shared::prepare_headers(args_clone.headers.clone());

    // Start measuring the request duration
    let start_time = Instant::now();

    let url = match &args_clone.url {
        Some(u) => u,
        None => {
            eprintln!("❌ Error: URL is missing.");
            return;
        }
    };

    let res = match args_clone.method {
        Some(HTTPMethods::GET) | None => match client
            .get(url)
            .headers(headers_hash)
            .send()
            .await
            {
                Ok(body) => body,
                Err(_) => return,
            },
        Some(HTTPMethods::POST) => match client
            .post(url)
            .headers(headers_hash)
            .body(args_clone.data.as_str().to_owned())
            .send()
            .await
            {
                Ok(body) => body,
                Err(_) => return,
            }
    };

    // Measure the duration
    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis();

    let status_code = res.status();

    if let Ok(body) = res.text().await {
        if !filter_response(status_code.into(), &body, body.len(), 0, &args_clone) {
            let size = body.len();
            let words = body.split_whitespace().count();
            let lines = body.lines().count();

            // Print formatted output
            println!("\r{:<24} [Status: {}, Size: {}, Words: {}, Lines: {}, Duration: {}ms]", word, status_code.as_u16(), size, words, lines, duration_ms);
        }
    }
}

async fn open_file(path: &PathBuf) -> tokio::io::Lines<tokio::io::BufReader<tokio::fs::File>> {
    match tokio::fs::File::open(path).await {
        Ok(file) => {
            let reader = tokio::io::BufReader::new(file);
            reader.lines()
        },
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }

    }

}

fn client_craft() -> Arc<Client> {
    match Client::builder().timeout(Duration::from_secs(1)).build() {
        Ok(client) => {
            Arc::new(client)
        },
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}



fn progress(counter_clone: Arc<AtomicUsize>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            print!(
                "\rWords processed so far: {}",
                counter_clone.load(Ordering::Relaxed)
            );
            std::io::Write::flush(&mut std::io::stdout()).unwrap(); // force immediate flush
            sleep(Duration::from_millis(500)).await;
        }
    })
}

pub async fn safe_buster(args: cli::Args) -> tokio::io::Result<()> {
    // Counter variables
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_cloned = Arc::clone(&counter);

    let semaphore = Arc::new(Semaphore::new(args.concurrent_tasks));

    // Craft the client
    let client = client_craft();

    // Open the wordlist file
    let mut lines = open_file(&args.wordlist).await;

    // Handler for the threads
    let mut tasks = JoinSet::new();
  

    let progress_task = progress(counter_cloned);

    while let Some(word) = lines.next_line().await? {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let args = args.clone();
        let client = Arc::clone(&client);
        counter.fetch_add(1, Ordering::Relaxed);

        tasks.spawn(async move {
            let _permit = permit; // Keeps the semaphore permit alive
            craft_request(args, client, word).await;
        });
    }
    while(tasks.join_next().await).is_some(){};

    progress_task.abort();

    Ok(())
}

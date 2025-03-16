use std::path::PathBuf;

use clap::Parser;
mod buster;
#[tokio::main(flavor = "multi_thread", worker_threads = 20)]
async fn main() {
    let args = buster::cli::Args::parse();
    let file = PathBuf::from("/home/HOME/Desktop/rust/safebuster/test");
    let _ = buster::safebuster::parse_file(file, args);
    // let _ = buster::safebuster::safe_buster(args.clone()).await;
    // let _ = buster::safebuster::search_fuzz(args, "Something");
}

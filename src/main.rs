use clap::Parser;
use tokio::main;
mod buster;
#[tokio::main]
async fn main() {
    let args = buster::cli::Args::parse();
    let _ = buster::safebuster::safe_buster(args.clone()).await;
    // let _ = buster::safebuster::search_fuzz(args, "Something");
}

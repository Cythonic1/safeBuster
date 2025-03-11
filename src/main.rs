use clap::Parser;
mod buster;
fn main() {
    let args = buster::cli::Args::parse();
    buster::safebuster::safe_buster(args.clone());
    // let _ = buster::safebuster::search_fuzz(args, "Something");
}

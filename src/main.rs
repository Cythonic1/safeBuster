use clap::Parser;
mod buster;
fn main() {
    let args = buster::cli::Args::parse();
    let _sometyhiong = buster::safebuster::test();
    let new = String::from("From Yaser");
    let _something: i32 = 10;
    println!("Headers are {:?}", args.headers);

    println!("Hello, world!, {}", new);
}

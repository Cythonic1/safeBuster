
use clap::Parser;
mod buster;


#[tokio::main(flavor = "multi_thread", worker_threads = 20)]
async fn main() {
    let args = buster::cli::Args::parse();


    if args.url.clone().is_some(){
            println!("You are using the cli verstion");
            let _ = buster::safebuster::safe_buster(args.clone()).await;
    }else {
        if args.file.is_some() {
            let mut test = buster::filehandle::FileParsing::new(args);
            test.open_file();
            test.prepare_args_from_file();
            let _ = buster::safebuster::safe_buster(test.args.clone()).await;

        }else {
            panic!("No URL provided");
        }
    }


}

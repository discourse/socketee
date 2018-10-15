extern crate socketee;

use std::env;
use std::process;

use socketee::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    eprintln!("Relaying datagrams from {} to {}", config.srcpath, config.dstpath);

    if let Err(e) = socketee::run(config) {
        println!("Fatal error: {}", e);
        process::exit(1);
    }
}

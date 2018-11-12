#[macro_use] 
extern crate serde_derive;

extern crate messenger_parser;

use std::env;
use std::process;

use messenger_parser::Config;

fn main() {

    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = messenger_parser::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

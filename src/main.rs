extern crate messenger_parser;

use std::env;
use std::process;

use messenger_parser::Config;

fn main() {
    // run_using_cli();
    messenger_parser::server::launch_server();
}

#[allow(dead_code)]
fn run_using_cli() {
    let config = Config::new_cli(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let result = messenger_parser::run(config, None).unwrap_or_else(|err| {
        eprintln!("Application error: {}", err);
        process::exit(1);
    });

    println!("{}", result);
}

extern crate messenger_parser;

use std::env;
use std::process;

use messenger_parser::Config;

fn main() {
    let args = env::args();
    parse_launching_mode(args);
}

fn parse_launching_mode(mut args: env::Args) {
    args.next();

    let running_mode = match args.next() {
        Some(args) => args,
        None => {
            eprintln!("Problem parsing arguments");
            process::exit(1);
        }
    };

    if running_mode == "server" {
        messenger_parser::server::launch_server();    
    } else if running_mode == "cli" {
        run_using_cli();
    } else {
        eprintln!("Problem parsing arguments");
        process::exit(1);
    }
}

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

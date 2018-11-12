#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::env;
use std::fs;
use std::error::Error;
use std::process;

pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next();

        let filename = match args.next() {
            Some(args) => args,
            None => return Err("Didn't get a query string"),
        };

        Ok(Config { filename })
    }
}

#[derive(Deserialize, Debug)]
pub struct Messages {
    messages: Vec<Message>,
}

#[derive(Deserialize, Debug)]
pub struct Message {
  share: Option<Share>,
}

#[derive(Deserialize, Debug)]
pub struct Share {
  link: String,
}

fn parse_messages(filename: String) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;
    let v: Messages = serde_json::from_str(&contents).unwrap();
    for message in v.messages {
      if message.share.is_some() {
        println!("{}", message.share.unwrap().link);
      }
    }
    Ok(())
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if let Err(e) = parse_messages(config.filename) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }

    Ok(())
}

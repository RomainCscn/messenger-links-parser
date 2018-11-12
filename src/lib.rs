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
    pub site: Option<String>,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next();

        let filename = match args.next() {
            Some(args) => args,
            None => return Err("Didn't get a query string"),
        };

        let site = match args.next() {
          Some(args) => Some(args),
          None => None,
        };

        Ok(Config { filename, site })
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

fn parse_messages(filename: String, site: Option<String>) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;
    let v: Messages = serde_json::from_str(&contents).unwrap();
    if site.is_some() {
      let filter_site = site.unwrap();
      for message in v.messages {
        if message.share.is_some() {
          let link = message.share.unwrap().link;
          if link.contains(&filter_site) {
            println!("{}", link);
          }
        }
      }
    } else {
      for message in v.messages {
        if message.share.is_some() {
          println!("{}", message.share.unwrap().link);
        }
      }
    }
    Ok(())
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if let Err(e) = parse_messages(config.filename, config.site) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }

    Ok(())
}

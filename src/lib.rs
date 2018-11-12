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
pub struct JsonValue {
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

fn search_links_with_filter(messages: Vec<Message>, filter_site: String) -> Vec<String> {
    let mut links = Vec::new();
    for message in messages {
      if message.share.is_some() {
        let link = message.share.unwrap().link;
        if link.contains(&filter_site) {
          links.push(link);
        }
      }
    }
    links
}

fn search_links_without_filter(messages: Vec<Message>) -> Vec<String>{
    let mut links = Vec::new();
    for message in messages {
      if message.share.is_some() {
        links.push(message.share.unwrap().link);
      }
    }
    links
}

fn parse_messages(json_value: JsonValue, site: Option<String>) -> Result<(), Box<dyn Error>> {
    if site.is_some() {
      let filter_site = site.unwrap();
      search_links_with_filter(json_value.messages, filter_site);
    } else {
      search_links_without_filter(json_value.messages);
    }
    Ok(())
}

fn parse_file(filename: String) -> Result<JsonValue, Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;
    let value: JsonValue = serde_json::from_str(&contents).unwrap();
    Ok(value)
}
 
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let json_value = parse_file(config.filename).unwrap_or_else(|err| {
        eprintln!("Problem parsing file: {}", err);
        process::exit(1);
    });

    if let Err(e) = parse_messages(json_value, config.site) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_without_filter() {
        let share1 = Share {
          link: String::from("https://www.youtube.com/watch?v=aJUQO9l7k5s"),
        };
        let share2 = Share {
          link: String::from("https://www.youtube.com/watch?v=dazedazed"),
        };
        let message1 = Message {
          share: Some(share1),
        };
        let message2 = Message {
          share: None,
        };
        let message3 = Message {
          share: Some(share2),
        };

        assert_eq!(
            vec![String::from("https://www.youtube.com/watch?v=aJUQO9l7k5s"), String::from("https://www.youtube.com/watch?v=dazedazed")],
            search_links_without_filter(vec![message1, message2, message3])
        );
    }

    #[test]
    fn search_with_filter() {
        let share1 = Share {
          link: String::from("https://www.youtube.com/watch?v=aJUQO9l7k5s"),
        };
        let share2 = Share {
          link: String::from("https://www.reddit.com/r/france"),
        };
        let message1 = Message {
          share: Some(share1),
        };
        let message2 = Message {
          share: None,
        };
        let message3 = Message {
          share: Some(share2),
        };

        assert_eq!(
            vec![String::from("https://www.reddit.com/r/france")],
            search_links_with_filter(vec![message1, message2, message3], String::from("reddit"))
        );
    }
}
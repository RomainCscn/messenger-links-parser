#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;

use std::env;
use std::error::Error;
use std::process;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::LineWriter;

use chrono::TimeZone;
use chrono::Utc;

pub mod server;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub filename: String,
    pub site: Option<String>,
    pub sender: Option<String>,
}

impl Config {
    pub fn new_cli(mut args: env::Args) -> Result<Config, &'static str> {
        args.next();

        let filename = match args.next() {
            Some(args) => args,
            None => return Err("Didn't get a query string"),
        };

        let site = match args.next() {
          Some(args) => Some(args),
          None => None,
        };

        let sender = match env::var("SENDER") {
          Ok(value) => Some(value),
          Err(_e) => None,
        };

        Ok(Config { filename, site, sender })
    }

    pub fn new(filename: String, site: Option<String>, sender: Option<String>) -> Result<Config, &'static str> {
      Ok(Config { filename, site, sender })
    }
}

#[derive(Deserialize, Debug)]
pub struct JsonValue {
    messages: Vec<Message>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Message {
  sender_name: String,
  content: String,
  timestamp_ms: i64,
  share: Option<Share>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Share {
  link: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct LinkInfo {
  sender_name: String,
  date: String,
  link: String,
}

#[derive(Serialize, Debug, )]
pub struct JsonExport<'a> {
  links: &'a Vec<LinkInfo>,
}

fn create_link_info(sender_name: String, timestamp: i64, link: String) -> LinkInfo {
   let dt = Utc.timestamp_millis(timestamp);

   let link_info = LinkInfo {
      sender_name,
      date: dt.format("%d-%m-%Y %H:%M:%S").to_string(),
      link,
    };
    link_info
}

fn search_links_with_filter(messages: &[Message], filter_site: String) -> Vec<LinkInfo> {
    let mut links_info = Vec::new();
    for message in messages.iter().cloned() {
      if message.share.is_some() {
        let link = message.share.unwrap().link;
        if link.contains(&filter_site) {
          let link_info = create_link_info(message.sender_name, message.timestamp_ms, link);
          links_info.push(link_info);
        }
      } else if message.content.contains("http://") || message.content.contains("https://") {
        let v: Vec<&str> = message.content.split_whitespace().collect();
        for word in v {
          if (word.contains("http://") || word.contains("https://")) && word.contains(&filter_site) {
            let link_info = create_link_info((message.sender_name).to_string(), message.timestamp_ms, word.to_string());
            links_info.push(link_info);
          }
        }
      }
    }
    links_info
}

fn search_links_without_filter(messages: &[Message]) -> Vec<LinkInfo> {
    let mut links_info = Vec::new();
    for message in messages.iter().cloned() {
      if message.share.is_some() {
        let link = message.share.unwrap().link;
        let link_info = create_link_info(message.sender_name, message.timestamp_ms,link);
        links_info.push(link_info);
      } else if message.content.contains("http://") || message.content.contains("https://") {
        let v: Vec<&str> = message.content.split_whitespace().collect();
        for word in v {
          if word.contains("http://") || word.contains("https://") {
            let link_info = create_link_info((message.sender_name).to_string(), message.timestamp_ms, word.to_string());
            links_info.push(link_info);
          }
        }
      }
    }
    links_info
}

fn filter_sender(messages: Vec<Message>, sender: &str) -> Vec<Message> {
  let mut sender_messages = Vec::new();
  for message in messages {
    if message.sender_name.contains(sender) {
      sender_messages.push(message);
    }
  }
  sender_messages
}

fn write_json_file(content: String) -> std::io::Result<()> {
  let file = File::create("test.json")?;
  let mut file = LineWriter::new(file);
  file.write_all(content.into_bytes().as_slice())?;

  Ok(())
}

fn return_links_json(links_info: &Vec<LinkInfo>) -> Result<(String), Box<dyn Error>> {
    let json_to_export = JsonExport {
      links: links_info,
    };
    
    let json = serde_json::to_string_pretty(&json_to_export)?;
    // write_json_file(json)?;
    Ok(json)
}

fn parse_messages(json_value: JsonValue, site: Option<String>, sender: Option<String>) -> Result<(String), Box<dyn Error>> {
    let json;
    if site.is_some() {
      let filter_site = site.unwrap();
      if sender.is_some() {
        let messages = filter_sender(json_value.messages, sender.unwrap().as_str());
        json = return_links_json(&search_links_with_filter(&messages, filter_site))?;
      } else {
        json = return_links_json(&search_links_with_filter(&json_value.messages, filter_site))?;
      }
    } else {
      if sender.is_some() {
        let messages = filter_sender(json_value.messages, sender.unwrap().as_str());
        json = return_links_json(&search_links_without_filter(&messages))?;
      } else {
        json = return_links_json(&search_links_without_filter(&json_value.messages))?;
      }
    }
    Ok(json)
}

fn parse_file(filename: String) -> Result<JsonValue, Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;
    let value: JsonValue = serde_json::from_str(&contents).unwrap();
    Ok(value)
}
 
pub fn run(config: Config) -> Result<(String), Box<dyn Error>> {
    let json_value = parse_file(config.filename).unwrap_or_else(|err| {
        eprintln!("Problem parsing file: {}", err);
        process::exit(1);
    });

    let json_links = parse_messages(json_value, config.site, config.sender).unwrap_or_else(|err| {
        eprintln!("Application error: {}", err);
        process::exit(1);
    });

    Ok(json_links)
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
          sender_name: String::from("toto"),
          share: Some(share1),
          content: String::from(""),
          timestamp_ms: 122,
        };
        let message2 = Message {
          sender_name: String::from("toto"),
          share: None,
          content: String::from(""),
          timestamp_ms: 122,
        };
        let message3 = Message {
          sender_name: String::from("toto"),
          share: Some(share2),
          content: String::from(""),
          timestamp_ms: 122,
        };
        let link1 = LinkInfo {
          sender_name: String::from("toto"),
          date: Utc.timestamp_millis(122).format("%d-%m-%Y %H:%M:%S").to_string(),
          link: String::from("https://www.youtube.com/watch?v=aJUQO9l7k5s"),
        };
        let link2 = LinkInfo {
          sender_name: String::from("toto"),
          date: Utc.timestamp_millis(122).format("%d-%m-%Y %H:%M:%S").to_string(),
          link: String::from("https://www.youtube.com/watch?v=dazedazed"),
        };

        assert_eq!(
            vec![link1, link2],
            search_links_without_filter(&vec![message1, message2, message3])
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
          sender_name: String::from("toto"),
          share: Some(share1),
          content: String::from(""),
          timestamp_ms: 122,
        };
        let message2 = Message {
          sender_name: String::from("toto"),
          share: None,
          content: String::from(""),
          timestamp_ms: 122,
        };
        let message3 = Message {
          sender_name: String::from("toto"),
          share: Some(share2),
          content: String::from(""),
          timestamp_ms: 122,
        };
        let link1 = LinkInfo {
          sender_name: String::from("toto"),
          date: Utc.timestamp_millis(122).format("%d-%m-%Y %H:%M:%S").to_string(),
          link: String::from("https://www.reddit.com/r/france"),
        };

        assert_eq!(
            vec![link1],
            search_links_with_filter(&vec![message1, message2, message3], String::from("reddit"))
        );
    }
}
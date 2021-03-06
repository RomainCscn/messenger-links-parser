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
use chrono::prelude::*;

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
pub struct DateFilter {
  pub year: Option<i32>,
  pub month: Option<u32>,
  pub day: Option<u32>
}

#[derive(Deserialize, Debug)]
pub struct JsonValue {
    messages: Vec<Message>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Message {
  sender_name: String,
  content: Option<String>,
  timestamp_ms: i64,
  share: Option<Share>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Share {
  link: Option<String>,
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

fn search_in_share(message: Message) -> Option<LinkInfo> {
  let share = message.share.unwrap();
  if share.link.is_some() {
    let link_info = create_link_info(message.sender_name, message.timestamp_ms, share.link.unwrap());
    return Some(link_info);
  }
  return None;
}

fn search_in_content(message: Message) -> Option<LinkInfo> {
  let content = message.content.unwrap();
  let v: Vec<&str> = content.split_whitespace().collect();
  for word in v {
    if word.contains("http://") || word.contains("https://") {
      let link_info = create_link_info((message.sender_name).to_string(), message.timestamp_ms, word.to_string());
      return Some(link_info)
    }
  }
  return None;
}

fn search_links_with_site_filter(messages: &[Message], filter_site: String) -> Vec<LinkInfo> {
  let mut links_info = Vec::new();
  for message in messages.iter().cloned() {
    if message.share.is_some() {
      if let Some(link_info) = search_in_share(message) {
        if link_info.link.contains(&filter_site) {
          links_info.push(link_info);
        }
      }
    } else if message.content.is_some() && (message.clone().content.unwrap().contains("http://") || message.clone().content.unwrap().contains("https://")) {
        if let Some(link_info) = search_in_content(message) {
          if link_info.link.contains(&filter_site) {
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
      if let Some(link_info) = search_in_share(message) {
        links_info.push(link_info);
      }
    } else if message.content.is_some() && (message.clone().content.unwrap().contains("http://") || message.clone().content.unwrap().contains("https://")) {
        if let Some(link_info) = search_in_content(message) {
          links_info.push(link_info);
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

fn filter_date(messages: Vec<Message>, date_filter: DateFilter) -> Vec<Message> {
    let has_year = date_filter.year.is_some();
    let has_month = date_filter.month.is_some();
    let has_day = date_filter.day.is_some();
    
    let mut date_messages = Vec::new();
    for message in messages {
      let date = Utc.timestamp_millis(message.timestamp_ms);
      if has_year {
        if has_month && has_day {
          if date.year() == date_filter.year.unwrap() && date.month() == date_filter.month.unwrap() && date.day() == date_filter.day.unwrap() {
            date_messages.push(message);
          }
        } else if has_month {
          if date.year() == date_filter.year.unwrap() && date.month() == date_filter.month.unwrap() {
            date_messages.push(message);
          }
        } else if has_day {
          if date.year() == date_filter.year.unwrap() && date.day() == date_filter.day.unwrap() {
            date_messages.push(message);
          }
        } else if date.year() == date_filter.year.unwrap() {
            date_messages.push(message);
          }
        }
      }
    date_messages
}

#[allow(dead_code)]
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

fn parse_messages(json_value: JsonValue, config: Config, date_filter: Option<DateFilter>) -> Result<(String), Box<dyn Error>> {
    let json;
    let mut messages = json_value.messages;
    let has_date_filter = date_filter.is_some();
    if config.site.is_some() {
      let filter_site = config.site.unwrap();
      if config.sender.is_some() {
        messages = filter_sender(messages, config.sender.unwrap().as_str());
      }
      if has_date_filter {
        messages = filter_date(messages, date_filter.unwrap());
      }
      json = return_links_json(&search_links_with_site_filter(&messages, filter_site))?;
    } else {
      if config.sender.is_some() {
        messages = filter_sender(messages, config.sender.unwrap().as_str());
      }
      if has_date_filter {
        messages = filter_date(messages, date_filter.unwrap());
      }
      json = return_links_json(&search_links_without_filter(&messages))?;
    }
    Ok(json)
}

fn parse_file(filename: String) -> Result<JsonValue, Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;
    let value: JsonValue = serde_json::from_str(&contents).unwrap();
    Ok(value)
}
 
pub fn run(config: Config, date_filter: Option<DateFilter>) -> Result<(String), Box<dyn Error>> {
    let filename = config.filename.to_string();
    let json_value = parse_file(filename).unwrap_or_else(|err| {
        eprintln!("Problem parsing file: {}", err);
        process::exit(1);
    });

    let json_links = parse_messages(json_value, config, date_filter).unwrap_or_else(|err| {
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
          link: Some(String::from("https://www.youtube.com/watch?v=aJUQO9l7k5s")),
        };
        let share2 = Share {
          link: Some(String::from("https://www.youtube.com/watch?v=dazedazed")),
        };
        let message1 = Message {
          sender_name: String::from("toto"),
          share: Some(share1),
          content: Some(String::from("")),
          timestamp_ms: 122,
        };
        let message2 = Message {
          sender_name: String::from("toto"),
          share: None,
          content: Some(String::from("")),
          timestamp_ms: 122,
        };
        let message3 = Message {
          sender_name: String::from("toto"),
          share: Some(share2),
          content: Some(String::from("")),
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
          link: Some(String::from("https://www.youtube.com/watch?v=aJUQO9l7k5s")),
        };
        let share2 = Share {
          link: Some(String::from("https://www.reddit.com/r/france")),
        };
        let message1 = Message {
          sender_name: String::from("toto"),
          share: Some(share1),
          content: Some(String::from("")),
          timestamp_ms: 122,
        };
        let message2 = Message {
          sender_name: String::from("toto"),
          share: None,
          content: Some(String::from("")),
          timestamp_ms: 122,
        };
        let message3 = Message {
          sender_name: String::from("toto"),
          share: Some(share2),
          content: Some(String::from("")),
          timestamp_ms: 122,
        };
        let link1 = LinkInfo {
          sender_name: String::from("toto"),
          date: Utc.timestamp_millis(122).format("%d-%m-%Y %H:%M:%S").to_string(),
          link: String::from("https://www.reddit.com/r/france"),
        };

        assert_eq!(
            vec![link1],
            search_links_with_site_filter(&vec![message1, message2, message3], String::from("reddit"))
        );
    }

    #[test]
    fn filter_sender_test() {
      let message1 = Message {
          sender_name: String::from("toto"),
          share: None,
          content: Some(String::from("")),
          timestamp_ms: 122,
        };
        let message2 = Message {
          sender_name: String::from("foo"),
          share: None,
          content: Some(String::from("")),
          timestamp_ms: 122,
        };
        let message3 = Message {
          sender_name: String::from("toto"),
          share: None,
          content: Some(String::from("")),
          timestamp_ms: 122,
        };

        assert_eq!(
          vec![message1.clone(), message3.clone()],
          filter_sender(vec![message1, message2, message3], "toto")
        );
    }

    #[test]
    fn filter_date_test() {
      let message1 = Message {
          sender_name: String::from("toto"),
          share: None,
          content: Some(String::from("")),
          timestamp_ms: 1518370388967,
        };
        let message2 = Message {
          sender_name: String::from("foo"),
          share: None,
          content: Some(String::from("")),
          timestamp_ms: 1433346000259,
        };
        let message3 = Message {
          sender_name: String::from("toto"),
          share: None,
          content: Some(String::from("")),
          timestamp_ms: 1518370850974,
        };
        let date_filter = DateFilter {
          year: Some(2018),
          month: Some(2),
          day: Some(11)
        };

        assert_eq!(
          vec![message1.clone(), message3.clone()],
          filter_date(vec![message1, message2, message3], date_filter)
        );
    }

    #[test]
    fn create_link_info_test() {
      let link = LinkInfo {
          sender_name: String::from("toto"),
          date: Utc.timestamp_millis(122).format("%d-%m-%Y %H:%M:%S").to_string(),
          link: String::from("https://www.youtube.com/watch?v=aJUQO9l7k5s"),
      };

      assert_eq!(
        link, 
        create_link_info(String::from("toto"), 122, String::from("https://www.youtube.com/watch?v=aJUQO9l7k5s")));
    }
}
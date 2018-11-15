extern crate actix_web;
extern crate listenfd;

use self::listenfd::ListenFd;
use self::actix_web::{server, App, Json, HttpRequest, Path, Result, http};

use super::{Config, run};

#[derive(Deserialize, Debug)]
struct Info {
    filename: String,
    site: Option<String>,
    sender: Option<String>
}

fn search_all(req: &HttpRequest) -> Result<String> {
    let config = Config::new(String::from("message.json"), None, None).unwrap();
    let result = run(config).unwrap();
    Ok(format!("{}", result))
}

/// extract path info from "/sender/{sender}" url
/// {sender} - deserializes to a String
fn search_site(info: Path<(String)>) -> Result<String> {
    let config = Config::new(String::from("message.json"), Some(info.to_string()), None).unwrap();
    let result = run(config).unwrap();
    Ok(format!("{}", result))
}

// extract path info from "/sender/{sender}" url
// {sender} - deserializes to a String
fn search_sender(info: Path<(String)>) -> Result<String> {
    let config = Config::new(String::from("message.json"), None, Some(info.to_string())).unwrap();
    let result = run(config).unwrap();
    Ok(format!("{}", result))
}

fn search_site_and_sender(info: Path<(String, String)>) -> Result<String> {
    let config = Config::new(String::from("message.json"), Some(info.0.to_string()), Some(info.1.to_string())).unwrap();
    let result = run(config).unwrap();
    Ok(format!("{}", result))
}

pub fn launch_server() {
    let mut listenfd = ListenFd::from_env();

    let mut server = server::new(|| {
        App::new()
            .prefix("/search")
            .resource("/all", |r| r.method(http::Method::GET).f(search_all))
            .resource(
                "/sender/{sender}",                    
                |r| r.method(http::Method::GET).with(search_sender))
            .resource(
                "/site/{site}",                    
                |r| r.method(http::Method::GET).with(search_site))
            .resource(
                "/site/{site}/sender/{sender}",                    
                |r| r.method(http::Method::GET).with(search_site_and_sender))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)
    } else {
        server.bind("127.0.0.1:3000").unwrap()
    };

    server.run();
}
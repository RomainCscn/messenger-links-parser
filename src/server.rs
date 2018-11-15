extern crate actix;
extern crate actix_web;
extern crate listenfd;
extern crate env_logger;

use self::actix_web::{server, App, HttpRequest, Path, Result, http::{header, Method}, middleware, middleware::cors::Cors};

use std::env;

use super::{Config, run};

#[derive(Deserialize, Debug)]
struct Info {
    filename: String,
    site: Option<String>,
    sender: Option<String>
}

fn search_all(_req: &HttpRequest) -> Result<String> {
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
    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let sys = actix::System::new("Actix-web-CORS");

    server::new(move || {
        App::new()
            .prefix("/search")
            .middleware(middleware::Logger::default())
            .configure(|app| {
                Cors::for_app(app)
                    .allowed_origin("http://localhost:8080")
                    .allowed_methods(vec!["GET"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600)
                    .resource("/all", |r| r.method(Method::GET).f(search_all))
                    .resource(
                        "/sender/{sender}",                    
                        |r| r.method(Method::GET).with(search_sender))
                    .resource(
                        "/site/{site}",                    
                        |r| r.method(Method::GET).with(search_site))
                    .resource(
                        "/site/{site}/sender/{sender}",                    
                        |r| r.method(Method::GET).with(search_site_and_sender))
                    .register()
            })
    }).bind("127.0.0.1:3000")
        .unwrap()
        .shutdown_timeout(2)
        .start();

    let _ = sys.run();
}
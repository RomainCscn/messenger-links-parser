extern crate actix;
extern crate actix_web;
extern crate listenfd;
extern crate env_logger;

use self::actix_web::{server, App, Query, Path, Result, http::{header, Method}, middleware, middleware::cors::Cors};

use std::env;

use super::{Config, run, DateFilter};

fn run_config(config: Config, query: Query<DateFilter>) -> String {
    let result;
    if query.year.is_some() || query.month.is_some() || query.day.is_some() {
        result = run(config, Some(query.into_inner())).unwrap();
    } else {
        result = run(config, None).unwrap();
    }
    result
}

fn search_all(query: Query<DateFilter>) -> Result<String> {
    let config = Config::new(String::from("message.json"), None, None).unwrap();
    Ok(format!("{}", run_config(config, query)))
}

/// extract path info from "/sender/{sender}" url
/// {sender} - deserializes to a String
fn search_site((path, query): (Path<(String)>, Query<DateFilter>)) -> Result<String> {
    let config = Config::new(String::from("message.json"), Some(path.to_string()), None).unwrap();
    Ok(format!("{}", run_config(config, query)))
}

// extract path info from "/sender/{sender}" url
// {sender} - deserializes to a String
fn search_sender((path, query): (Path<(String)>, Query<DateFilter>)) -> Result<String> {
    let config = Config::new(String::from("message.json"), None, Some(path.to_string())).unwrap();
    Ok(format!("{}", run_config(config, query)))
}

fn search_site_and_sender((path, query): (Path<(String, String)>, Query<DateFilter>)) -> Result<String> {
    let config = Config::new(String::from("message.json"), Some(path.0.to_string()), Some(path.1.to_string())).unwrap();
    Ok(format!("{}", run_config(config, query)))
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
                    .resource(
                        "/all", 
                        |r| r.method(Method::GET).with(search_all))
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
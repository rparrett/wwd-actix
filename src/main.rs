#[macro_use]
extern crate log;
extern crate env_logger;

use actix_files as fs;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};

mod config;
mod filters;
mod forecast;
mod handler;

pub struct AppState {
    forecasts: Vec<forecast::BasicWeekendForecast>,
    last_fetch: Option<DateTime<Utc>>,
}

fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "info,actix_web=info");
    env_logger::init();

    let config = config::Config::new("config.toml").expect("Failed to open config file.");
    let addr = config.http.unwrap().addr.unwrap();

    let state = Arc::new(Mutex::new(AppState {
        forecasts: Vec::new(),
        last_fetch: None,
    }));

    let mut grabber = forecast::ForecastGrabber::new(
        state.clone(),
        config.darksky.unwrap(),
        config.locations.unwrap(),
    );

    // start http server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(state.clone())
            .service(fs::Files::new("/static", "static"))
            .service(web::resource("/").route(web::get().to(handler::index)))
    })
    .bind(addr)?
    .run();

    grabber.exit_join();

    Ok(())
}

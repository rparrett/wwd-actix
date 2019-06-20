extern crate serde;
extern crate toml;

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HttpConfig {
    pub addr: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DarkskyConfig {
    pub secret: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub http: Option<HttpConfig>,
    pub darksky: Option<DarkskyConfig>,
    pub locations: Option<Vec<Location>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub link: String,
}

impl Config {
    pub fn new(filename: &str) -> Result<Config, Box<Error>> {
        let mut input = String::new();

        File::open(filename)?.read_to_string(&mut input)?;

        let config = toml::from_str(&input)?;

        Ok(config)
    }
}

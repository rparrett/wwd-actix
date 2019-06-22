extern crate darksky;
extern crate reqwest;
extern crate serde;

use chrono::{DateTime, Datelike, TimeZone, Utc, Weekday};
use darksky::DarkskyReqwestRequester;
use reqwest::Client;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::config::DarkskyConfig;
use crate::config::Location;
use crate::AppState;

pub struct ForecastGrabber {
    pub exit: Arc<AtomicBool>,
    pub worker: Option<thread::JoinHandle<()>>,
}
impl ForecastGrabber {
    pub fn new(
        state: Arc<Mutex<AppState>>,
        config: DarkskyConfig,
        locations: Vec<Location>,
    ) -> ForecastGrabber {
        let exit = Arc::new(AtomicBool::new(false));
        let worker_exit = exit.clone();

        let worker = Some(thread::spawn(move || {
            let mut last_fetch: Option<DateTime<Utc>> = None;
            let secret = config.secret.unwrap();

            'outer: loop {
                if worker_exit.load(Ordering::Relaxed) {
                    info!("Worker received exit signal");
                    break;
                }

                let fetch_every = 60;

                let minutes = match last_fetch {
                    Some(d) => Utc::now().signed_duration_since(d).num_minutes(),
                    None => fetch_every,
                };

                if minutes >= fetch_every {
                    last_fetch = Some(Utc::now());

                    let mut failed = false;
                    let mut forecasts = Vec::new();

                    for location in &locations {
                        info!("Fetching: {}", location.name.clone());

                        match get_forecast(secret.clone(), location.lat, location.lon) {
                            Ok(forecast) => {
                                forecasts.push(BasicWeekendForecast {
                                    location: location.clone(),
                                    days: forecast,
                                });
                            }
                            Err(err) => {
                                info!("Error fetching: {}", err);

                                failed = true;
                                break;
                            }
                        }

                        if worker_exit.load(Ordering::Relaxed) {
                            info!("Worker received exit signal");
                            break 'outer;
                        }
                    }

                    if !failed {
                        let mut s = state.lock().unwrap();
                        s.forecasts = forecasts;
                        s.last_fetch = last_fetch;
                    }
                }

                std::thread::sleep(Duration::from_secs(1));
            }
        }));

        ForecastGrabber { exit, worker }
    }

    pub fn exit_join(&mut self) {
        info!("Sending exit signal to worker");

        self.exit.store(true, Ordering::Relaxed);
        if self.worker.take().unwrap().join().is_err() {
            error!("Worker thread paniced");
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BasicWeekendForecast {
    pub location: Location,
    pub days: Vec<BasicWeather>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BasicWeather {
    pub time: String,
    pub temperature_low: f64,
    pub temperature_high: f64,
    pub summary: String,
}

pub fn get_forecast(secret: String, lat: f64, long: f64) -> Result<Vec<BasicWeather>, String> {
    let client = Client::builder().build().or_else(|e| {
        Err(format!(
            "{} ({})",
            "Failed to build client".to_string(),
            e.to_string()
        ))
    })?;

    let req = client.get_forecast(&secret, lat, long).or_else(|e| {
        Err(format!(
            "{} ({})",
            "Request failed".to_string(),
            e.to_string()
        ))
    })?;

    let mut weathers = Vec::new();

    #[allow(clippy::identity_conversion)]
    for d in req
        .daily
        .ok_or_else(|| "No daily forecast".to_string())?
        .data
        .ok_or_else(|| "No Data".to_string())?
    {
        let dt = Utc.timestamp(d.time as i64, 0);

        match dt.weekday() {
            Weekday::Fri | Weekday::Sat | Weekday::Sun => {}
            _ => continue,
        }

        if d.temperature_low.is_none() || d.temperature_high.is_none() || d.summary.is_none() {
            continue;
        }

        let weather = BasicWeather {
            time: dt.format("%a %h %e").to_string(),
            temperature_low: d.temperature_low.unwrap(),
            temperature_high: d.temperature_high.unwrap(),
            summary: d.summary.unwrap(),
        };

        weathers.push(weather);
    }

    Ok(weathers)
}

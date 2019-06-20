use actix_web::{web, HttpResponse, Result};
use askama::Template;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::filters;
use crate::forecast;
use crate::AppState;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    forecasts: &'a Vec<forecast::BasicWeekendForecast>,
    last_fetch: String,
}

pub fn time_diff_in_words(time: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let diff = now.signed_duration_since(time);

    let minutes = diff.num_minutes();
    let hours = diff.num_hours();

    if minutes < 1 {
        "less than 1 minute".to_string()
    } else if minutes == 1 {
        "1 minute".to_string()
    } else if minutes < 60 {
        format!("{} minutes", minutes)
    } else if hours == 1 {
        "1 hour".to_string()
    } else {
        format!("{} hours", hours)
    }
}

pub fn index(
    state: web::Data<Arc<Mutex<AppState>>>,
    _query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let guard = state.lock().unwrap();

    let last_fetch = match guard.last_fetch {
        Some(time) => time_diff_in_words(time, Utc::now()),
        None => "an unknowable amount of time".to_string(),
    };

    let s = IndexTemplate {
        forecasts: &guard.forecasts,
        last_fetch,
    }
    .render()
    .unwrap();

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

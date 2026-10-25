use chrono::{NaiveDateTime, Utc};
use chrono_tz::Tz;
use serde::Serialize;

pub mod auth;
pub mod station;

#[derive(Debug, Serialize)]
pub struct ResponseBody<T> {
    message: String,
    success: bool,
    data: Option<T>,
}

impl<T> ResponseBody<T> {
    pub fn new(success: bool, message: String, data: Option<T>) -> Self {
        Self {
            message,
            success,
            data,
        }
    }
}

pub fn first_error(e: &validator::ValidationErrors) -> String {
    e.field_errors()
        .into_iter()
        .next()
        .and_then(|(field, errs)| {
            errs.first().map(|err| match &err.message {
                Some(msg) => msg.to_string(),
                None => format!("{} is invalid", field),
            })
        })
        .unwrap_or_else(|| "invalid request".to_string())
}

pub struct PaginationParams {
    pub page: i64,
    pub limit: i64,
    pub offset: i64,
}

impl PaginationParams {
    pub fn new(_page: Option<i64>, _limit: Option<i64>) -> Self {
        let page = _page.unwrap_or(1);
        let limit = _limit.unwrap_or(10).min(50);

        Self {
            page,
            limit,
            offset: (page - 1) * limit,
        }
    }
}

pub fn get_current_date_time(tz: &Tz) -> NaiveDateTime {
    Utc::now().with_timezone(tz).naive_local()
}

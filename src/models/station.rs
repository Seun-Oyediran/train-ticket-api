use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Station {
    id: Uuid,
    name: String,
    pub position: i32,
}

#[derive(Debug, Serialize)]
pub struct StationDates {
    dates: Vec<NaiveDate>,
    total_days: u8,
}

impl StationDates {
    pub fn new(dates: Vec<NaiveDate>, total_days: u8) -> Self {
        Self { dates, total_days }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetTripQuery {
    pub from: Uuid,
    pub to: Uuid,
    pub date: NaiveDate,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TripOption {
    pub schedule_id: Uuid,
    direction: String,
    pub departure_time: NaiveTime,
    pub boarding_time: NaiveTime,
    pub arrival_time: NaiveTime,
    classes: sqlx::types::Json<Vec<ClassAvailability>>,
}

#[derive(Debug, Serialize)]
pub struct GetTripResponse {
    from: Station,
    to: Station,
    trips: Vec<TripOption>,
}

impl GetTripResponse {
    pub fn new(from: Station, to: Station, trips: Vec<TripOption>) -> Self {
        Self { from, to, trips }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassAvailability {
    class_code: String,
    class_name: String,
    total_seats: i64,
    available_seats: i64,
    reserved_seats: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CoachAvailability {
    class_code: String,
    class_name: String,
    total_seats: i64,
    available_seats: i64,
    coach_number: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeatClass {
    First,
    Business,
    Standard,
}

impl SeatClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            SeatClass::Business => "business",
            SeatClass::First => "first",
            SeatClass::Standard => "standard",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetCoachesQuery {
    pub from: Uuid,
    pub to: Uuid,
    pub date: NaiveDate,
    pub class: SeatClass,
    pub schedule_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct GetSeatsQuery {
    pub from: Uuid,
    pub to: Uuid,
    pub date: NaiveDate,
    pub schedule_id: Uuid,
    pub coach: i32,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Seat {
    id: Uuid,
    coach_number: i32,
    seat_number: i32,
    seat_type: String,
    side: String,
    is_taken: bool,
}

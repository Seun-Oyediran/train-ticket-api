use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::models::pagination::PaginationResponse;

#[derive(Debug, Deserialize)]
pub struct BookTicketRequestBody {
    pub schedule_id: Uuid,
    pub from: Uuid,
    pub to: Uuid,
    pub date: NaiveDate,
    pub seat_id: Uuid,
}

#[derive(Debug, Serialize, FromRow)]
pub struct BookedTicket {
    id: Uuid,
    schedule_id: Uuid,
    from_station_id: Uuid,
    to_station_id: Uuid,
    seat_id: Uuid,
    travel_date: NaiveDate,
    status: String,
    from_station_name: String,
    to_station_name: String,
    class_code: String,
    class_name: String,
    seat_type: String,
    side: String,
    created_at: DateTime<Utc>,
    coach_number: i32,
    seat_number: i32,
}

#[derive(Debug, Serialize)]
pub struct BookingResponse {
    #[serde(flatten)]
    ticket: BookedTicket,
    departure_time: NaiveTime,
    boarding_time: NaiveTime,
    arrival_time: NaiveTime,
}

impl BookingResponse {
    pub fn new(
        ticket: BookedTicket,
        departure_time: NaiveTime,
        boarding_time: NaiveTime,
        arrival_time: NaiveTime,
    ) -> Self {
        Self {
            ticket,
            departure_time,
            boarding_time,
            arrival_time,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetTicketQuery {
    pub all: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UserTicket {
    id: Uuid,
    schedule_id: Uuid,
    from_station_id: Uuid,
    to_station_id: Uuid,
    seat_id: Uuid,
    status: String,
    from_station_name: String,
    to_station_name: String,
    class_code: String,
    class_name: String,
    seat_type: String,
    side: String,
    direction: String,
    travel_date: NaiveDate,
    created_at: DateTime<Utc>,
    coach_number: i32,
    seat_number: i32,
    departure_time: NaiveTime,
    boarding_time: NaiveTime,
    arrival_time: NaiveTime,
}

#[derive(Debug, Serialize)]
pub struct GetTicketsResponse {
    pagination: PaginationResponse,
    tickets: Vec<UserTicket>,
}

impl GetTicketsResponse {
    pub fn new(tickets: Vec<UserTicket>, pagination: PaginationResponse) -> Self {
        Self {
            pagination,
            tickets,
        }
    }
}

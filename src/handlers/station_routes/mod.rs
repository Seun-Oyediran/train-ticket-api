use axum::{Json, Router, extract::State, routing::get};
use chrono::NaiveDate;
use sqlx::PgPool;
use tokio::try_join;
use uuid::Uuid;

use crate::{
    error::AppError,
    extractors::AppQuery,
    handlers::{
        not_found::method_not_allowed,
        station_routes::queries::{
            fetch_coaches, fetch_seats_for_coach, fetch_station_by_id, fetch_train_for_trip,
            fetch_train_stations,
        },
    },
    models::station::{
        CoachAvailability, GetCoachesQuery, GetSeatsQuery, GetTripQuery, GetTripResponse, Seat,
        Station, StationDates, TripOption,
    },
    state::AppState,
    utils::{
        ResponseBody,
        station::{check_if_date_is_available, generate_days, get_current_time, get_today_date},
    },
};

mod queries;

pub fn station_routes_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_station_routes))
        .route("/dates", get(get_station_dates))
        .route("/trips", get(get_trips))
        .route("/coaches", get(get_coaches))
        .route("/seats", get(get_seats))
        .method_not_allowed_fallback(method_not_allowed)
}

pub async fn get_station_routes(
    State(state): State<AppState>,
) -> Result<Json<ResponseBody<Vec<Station>>>, AppError> {
    let stations = fetch_train_stations(&state.db).await?;

    Ok(Json(ResponseBody::new(
        true,
        "Successful".to_string(),
        Some(stations),
    )))
}

pub async fn get_station_dates(State(state): State<AppState>) -> Json<ResponseBody<StationDates>> {
    let days = generate_days(state.config.max_booking_days, state.config.train_timezone);

    Json(ResponseBody::new(
        true,
        "Successful".to_string(),
        Some(StationDates::new(days, state.config.max_booking_days)),
    ))
}

pub async fn get_trips(
    State(state): State<AppState>,
    AppQuery(query): AppQuery<GetTripQuery>,
) -> Result<Json<ResponseBody<GetTripResponse>>, AppError> {
    let (from_station, to_station, trips) =
        verify_trip(&state, query.from, query.to, query.date).await?;

    let data = GetTripResponse::new(from_station, to_station, trips);

    Ok(Json(ResponseBody::new(
        true,
        "Successful".to_string(),
        Some(data),
    )))
}

pub async fn get_coaches(
    State(state): State<AppState>,
    AppQuery(query): AppQuery<GetCoachesQuery>,
) -> Result<Json<ResponseBody<Vec<CoachAvailability>>>, AppError> {
    let (from_station, to_station, trips) =
        verify_trip(&state, query.from, query.to, query.date).await?;

    if !trips.iter().any(|f| f.schedule_id == query.schedule_id) {
        return Err(AppError::BadRequest(
            "This train is not available for the selected trip".to_string(),
        ));
    }

    let coaches = fetch_coaches(
        &state.db,
        query.schedule_id,
        from_station.position,
        to_station.position,
        query.date,
        query.class.as_str(),
    )
    .await?;

    Ok(Json(ResponseBody::new(
        true,
        "Successful".to_string(),
        Some(coaches),
    )))
}

pub async fn get_seats(
    State(state): State<AppState>,
    AppQuery(query): AppQuery<GetSeatsQuery>,
) -> Result<Json<ResponseBody<Vec<Seat>>>, AppError> {
    let (from_station, to_station, trips) =
        verify_trip(&state, query.from, query.to, query.date).await?;

    if !trips.iter().any(|f| f.schedule_id == query.schedule_id) {
        return Err(AppError::BadRequest(
            "This train is not available for the selected trip".to_string(),
        ));
    }

    let seats = fetch_seats_for_coach(
        &state.db,
        query.schedule_id,
        from_station.position,
        to_station.position,
        query.date,
        query.coach,
    )
    .await?;

    if seats.is_empty() {
        return Err(AppError::NotFound("Coach not found".to_string()));
    }

    Ok(Json(ResponseBody::new(
        true,
        "Successful".to_string(),
        Some(seats),
    )))
}

async fn check_if_stations_exist(
    db: &PgPool,
    from: Uuid,
    to: Uuid,
) -> Result<(Station, Station), AppError> {
    let (from_station_opt, to_station_opt) =
        try_join!(fetch_station_by_id(db, from), fetch_station_by_id(db, to))?;

    let from_station = from_station_opt
        .ok_or_else(|| AppError::NotFound("Entry station not found".to_string()))?;

    let to_station = to_station_opt
        .ok_or_else(|| AppError::NotFound("Destination station not found".to_string()))?;

    Ok((from_station, to_station))
}

pub async fn verify_trip(
    state: &AppState,
    from: Uuid,
    to: Uuid,
    date: NaiveDate,
) -> Result<(Station, Station, Vec<TripOption>), AppError> {
    if from == to {
        return Err(AppError::BadRequest(
            "Entry and destination must be different".to_string(),
        ));
    }

    check_if_date_is_available(
        date,
        state.config.train_timezone,
        state.config.max_booking_days,
    )?;

    let (from_station, to_station) = check_if_stations_exist(&state.db, from, to).await?;

    let mut trips =
        fetch_train_for_trip(&state.db, from_station.position, to_station.position, date).await?;

    if date == get_today_date(state.config.train_timezone) {
        let current_time = get_current_time(state.config.train_timezone);
        trips.retain(|item| current_time < item.boarding_time);
    }

    Ok((from_station, to_station, trips))
}

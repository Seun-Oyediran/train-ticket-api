use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use tokio::try_join;
use uuid::Uuid;

use crate::{
    error::AppError,
    extractors::{AppJson, AppPath, AppQuery},
    handlers::{
        not_found::method_not_allowed,
        station_routes::verify_trip,
        tickets::queries::{
            count_user_tickets, fetch_single_ticket, fetch_user_tickets, insert_book_ticket,
        },
    },
    models::{
        auth::Claims,
        pagination::{PaginationRequest, PaginationResponse},
        tickets::{
            BookTicketRequestBody, BookingResponse, GetTicketQuery, GetTicketsResponse, UserTicket,
        },
    },
    state::AppState,
    utils::{PaginationParams, ResponseBody},
};

mod queries;

pub fn ticket_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(book_ticket))
        .route("/", get(get_tickets))
        .route("/{id}", get(get_single_ticket))
        .method_not_allowed_fallback(method_not_allowed)
}

pub async fn book_ticket(
    claims: Claims,
    State(state): State<AppState>,
    AppJson(body): AppJson<BookTicketRequestBody>,
) -> Result<(StatusCode, Json<ResponseBody<BookingResponse>>), AppError> {
    let (_, _, trips) = verify_trip(&state, body.from, body.to, body.date).await?;

    let trip = trips
        .iter()
        .find(|f| f.schedule_id == body.schedule_id)
        .ok_or_else(|| {
            AppError::BadRequest("This train is not available for the selected trip".to_string())
        })?;

    let booked_ticket = insert_book_ticket(
        &state.db,
        claims.sub,
        body.from,
        body.to,
        body.schedule_id,
        body.seat_id,
        body.date,
    )
    .await?;

    let res = BookingResponse::new(
        booked_ticket,
        trip.departure_time,
        trip.boarding_time,
        trip.arrival_time,
    );

    Ok((
        StatusCode::CREATED,
        Json(ResponseBody::new(
            true,
            "Ticket booked successfully".to_string(),
            Some(res),
        )),
    ))
}

pub async fn get_tickets(
    claims: Claims,
    State(state): State<AppState>,
    AppQuery(query): AppQuery<GetTicketQuery>,
    AppQuery(pagination): AppQuery<PaginationRequest>,
) -> Result<Json<ResponseBody<GetTicketsResponse>>, AppError> {
    let pagination_params = PaginationParams::new(pagination.page, pagination.limit);
    let include_all = query.all.unwrap_or(false);

    let (tickets, count) = try_join!(
        fetch_user_tickets(
            &state.db,
            claims.sub,
            include_all,
            &state.config.train_timezone,
            pagination_params.limit,
            pagination_params.offset,
        ),
        count_user_tickets(
            &state.db,
            claims.sub,
            include_all,
            &state.config.train_timezone
        )
    )?;

    let pagination =
        PaginationResponse::new(count, pagination_params.limit, pagination_params.page);

    let response_data = GetTicketsResponse::new(tickets, pagination);
    let response = ResponseBody::new(true, String::from("Successful"), Some(response_data));

    Ok(Json(response))
}

pub async fn get_single_ticket(
    claims: Claims,
    State(state): State<AppState>,
    AppPath(ticket_id): AppPath<Uuid>,
) -> Result<Json<ResponseBody<UserTicket>>, AppError> {
    let ticket = fetch_single_ticket(&state.db, ticket_id, claims.sub)
        .await?
        .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    Ok(Json(ResponseBody::new(
        true,
        "Successful".to_string(),
        Some(ticket),
    )))
}

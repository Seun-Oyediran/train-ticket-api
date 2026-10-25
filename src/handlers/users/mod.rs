use axum::{Json, Router, extract::State, http::StatusCode, routing::get};

use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    extractors::{AppPath, AppQuery},
    handlers::{
        not_found::method_not_allowed,
        users::queries::{fetch_single_user, get_total_users, get_users_from_db},
    },
    models::{
        auth::Claims,
        pagination::{PaginationRequest, PaginationResponse},
        user::{User, UsersResponse},
    },
    state::AppState,
    utils::{PaginationParams, ResponseBody},
};

mod queries;

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_users))
        .route("/{id}", get(get_single_user))
        .method_not_allowed_fallback(method_not_allowed)
}

pub async fn get_users(
    State(state): State<AppState>,
    AppQuery(params): AppQuery<PaginationRequest>,
    _claims: Claims,
) -> Result<(StatusCode, Json<ResponseBody<UsersResponse>>), AppError> {
    params.validate()?;

    let pagination_params = PaginationParams::new(params.page, params.limit);

    let (users, count) = tokio::try_join!(
        get_users_from_db(&state.db, pagination_params.limit, pagination_params.offset),
        get_total_users(&state.db)
    )?;

    let pagination =
        PaginationResponse::new(count, pagination_params.limit, pagination_params.page);

    let response_data = UsersResponse::new(users, pagination);
    let response = ResponseBody::new(true, String::from("Successful"), Some(response_data));

    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_single_user(
    State(state): State<AppState>,
    AppPath(user_id): AppPath<Uuid>,
    _claims: Claims,
) -> Result<(StatusCode, Json<ResponseBody<User>>), AppError> {
    let user = fetch_single_user(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User does not exist".to_string()))?;

    let response = ResponseBody::new(true, String::from("Successful"), Some(user));
    Ok((StatusCode::OK, Json(response)))
}

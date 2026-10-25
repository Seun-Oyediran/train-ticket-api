use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Path, Query},
};

use crate::{error::AppError, models::auth::Claims, state::AppState};

#[derive(FromRequest)]
#[from_request(via(Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

#[derive(FromRequestParts)]
#[from_request(via(Query), rejection(AppError))]
pub struct AppQuery<T>(pub T);

#[derive(FromRequestParts)]
#[from_request(via(Path), rejection(AppError))]
pub struct AppPath<T>(pub T);

impl FromRequestParts<AppState> for Claims {
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let error_message = String::from("Unauthorized");

        let (prefix, token) = parts
            .headers
            .get("Authorization")
            .ok_or_else(|| AppError::Unauthorized(error_message.clone()))?
            .to_str()
            .map_err(|_| AppError::Unauthorized(error_message.clone()))?
            .split_once(' ')
            .ok_or_else(|| AppError::Unauthorized(error_message.clone()))?;

        if !prefix.eq_ignore_ascii_case("Bearer") {
            return Err(AppError::Unauthorized(error_message.clone()));
        }

        let claims = Claims::decode_jwt(token, &state.jwt_decoded_secret)
            .map_err(|_| AppError::Unauthorized(error_message))?;

        Ok(claims.claims)
    }
}

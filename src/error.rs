use argon2::password_hash::Error as Argon2Error;
use axum::{
    Json,
    extract::rejection::{JsonRejection, PathRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use jsonwebtoken::errors::Error as JsonWebTokenError;
use serde::Serialize;
use sqlx::Error as SqlxError;
use tower_governor::GovernorError;
use validator::ValidationErrors;

use crate::utils::first_error;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    message: String,
    success: bool,
}

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
    UnprocessableEntity(String),
    UnsupportedMediaType(String),
    Unauthorized(String),
    Conflict(String),
    MethodNotAllowed,
    PayloadTooLarge(String),
    TooManyRequests(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => {
                tracing::error!("server error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Server error".to_string(),
                )
            }
            AppError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::UnsupportedMediaType(msg) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::MethodNotAllowed => (
                StatusCode::METHOD_NOT_ALLOWED,
                "Method not allowed".to_string(),
            ),
            AppError::PayloadTooLarge(msg) => (StatusCode::PAYLOAD_TOO_LARGE, msg),
            AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
        };

        let error_body = Json(ErrorBody {
            message,
            success: false,
        });

        (status_code, error_body).into_response()
    }
}

impl From<JsonRejection> for AppError {
    fn from(value: JsonRejection) -> Self {
        match value {
            JsonRejection::BytesRejection(e) => {
                // Happens when there are issues buffering the request body (e.g., timeout, body too large)
                AppError::PayloadTooLarge(e.body_text())
            }
            JsonRejection::JsonDataError(e) => {
                // Happens when the JSON is valid, but fails validation or deserialization (e.g., missing fields)
                AppError::UnprocessableEntity(e.body_text())
            }
            JsonRejection::JsonSyntaxError(e) => {
                // Happens when the payload is malformed or invalid JSON syntax
                AppError::BadRequest(e.body_text())
            }
            JsonRejection::MissingJsonContentType(e) => {
                // Happens when the Client forgot to send `Content-Type: application/json`
                AppError::UnsupportedMediaType(e.body_text())
            }
            _ => AppError::Internal("Unknown JSON rejection error".to_string()),
        }
    }
}

impl From<QueryRejection> for AppError {
    fn from(value: QueryRejection) -> Self {
        AppError::BadRequest(value.body_text())
    }
}

impl From<PathRejection> for AppError {
    fn from(value: PathRejection) -> Self {
        AppError::BadRequest(value.body_text())
    }
}

impl From<SqlxError> for AppError {
    fn from(value: SqlxError) -> Self {
        match value {
            SqlxError::Database(e) => {
                match e.constraint() {
                    Some("users_email_key") => {
                        return AppError::Conflict("Email already in use".to_string());
                    }
                    Some("tickets_no_overlapping_seat") => {
                        return AppError::Conflict("Seat is no longer available".to_string());
                    }
                    Some("tickets_seat_id_fkey") => {
                        return AppError::NotFound("Seat not found".to_string());
                    }
                    Some("tickets_user_id_fkey") => {
                        return AppError::Unauthorized("Unauthorized".to_string());
                    }
                    _ => {}
                }

                if e.is_unique_violation() {
                    return AppError::Conflict("Resource already exists".to_string());
                }

                AppError::Internal(e.to_string())
            }
            e => AppError::Internal(e.to_string()),
        }
    }
}

impl From<ValidationErrors> for AppError {
    fn from(value: ValidationErrors) -> Self {
        let validation_error_message = first_error(&value);
        AppError::BadRequest(validation_error_message)
    }
}

impl From<JsonWebTokenError> for AppError {
    fn from(value: JsonWebTokenError) -> Self {
        let err_msg = format!("An error occured during encoding/decoding {}", value);
        AppError::Internal(err_msg)
    }
}

impl From<Argon2Error> for AppError {
    fn from(value: Argon2Error) -> Self {
        let err_msg = format!("Failed to hash or unhash {}", value);
        AppError::Internal(err_msg)
    }
}

impl From<GovernorError> for AppError {
    fn from(value: GovernorError) -> Self {
        match value {
            GovernorError::TooManyRequests {
                wait_time,
                headers: _,
            } => AppError::TooManyRequests(format!("Too many requests! Wait for {}s", wait_time)),

            _ => AppError::Internal("Server error".to_string()),
        }
    }
}

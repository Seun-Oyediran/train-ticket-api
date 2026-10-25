use crate::error::AppError;

pub async fn not_found() -> AppError {
    AppError::NotFound("Resource not found".to_string())
}

pub async fn method_not_allowed() -> AppError {
    AppError::MethodNotAllowed
}

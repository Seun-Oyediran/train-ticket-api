use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models::pagination::PaginationResponse;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    first_name: String,
    last_name: String,
    pub email: String,
    phone_number: String,
    date_of_birth: NaiveDate,
    gender: String,
    is_verified: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    verification_number: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserWithPassword {
    #[sqlx(flatten)]
    pub user: User,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UsersResponse {
    pagination: PaginationResponse,
    users: Vec<User>,
}

impl UsersResponse {
    pub fn new(users: Vec<User>, pagination: PaginationResponse) -> Self {
        Self { pagination, users }
    }
}

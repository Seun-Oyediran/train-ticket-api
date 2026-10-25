use sqlx::{Error as SqlxError, PgPool};
use uuid::Uuid;

use crate::models::user::User;

pub async fn fetch_single_user(db: &PgPool, id: Uuid) -> Result<Option<User>, SqlxError> {
    sqlx::query_as::<_, User>(
        r#"
    SELECT id, first_name, last_name, email, phone_number, date_of_birth,
           verification_number, gender, is_verified, created_at, updated_at
    FROM users
    WHERE id = $1
    "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn get_users_from_db(
    db: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<User>, SqlxError> {
    sqlx::query_as::<_, User>(
        r#"
    SELECT id, first_name, last_name, email, phone_number, date_of_birth,
           verification_number, gender, is_verified, created_at, updated_at
    FROM users
    ORDER BY created_at DESC
    LIMIT $1 OFFSET $2
    "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
}

pub async fn get_total_users(db: &PgPool) -> Result<i64, SqlxError> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(db)
        .await
}

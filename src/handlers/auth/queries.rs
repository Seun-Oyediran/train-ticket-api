use sqlx::{Error as SqlxError, PgPool};
use uuid::Uuid;

use crate::models::{
    auth::CreateAccountPayload,
    user::{User, UserWithPassword},
};

pub async fn find_user_with_password(
    db: &PgPool,
    email: String,
) -> Result<Option<UserWithPassword>, SqlxError> {
    sqlx::query_as::<_, UserWithPassword>(
        r#"
        SELECT id, first_name, last_name, email, phone_number, password, date_of_birth,
               verification_number, gender, is_verified, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(db)
    .await
}

pub async fn create_user(
    db: &PgPool,
    body: CreateAccountPayload,
    hashed_password: String,
) -> Result<User, SqlxError> {
    sqlx::query_as::<_, User>(
    r#"
    INSERT INTO users (first_name, last_name, email, phone_number, password, date_of_birth, gender, verification_number)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    RETURNING id, first_name, last_name, email, phone_number, date_of_birth, verification_number, gender,
              is_verified, created_at, updated_at
    "#,
)
.bind(&body.first_name)
.bind(&body.last_name)
.bind(&body.email)
.bind(&body.phone_number)
.bind(&hashed_password)
.bind(body.date_of_birth)
.bind(body.gender.as_str())
.bind(&body.verification_number)
.fetch_one(db)
.await
}

pub async fn store_refresh_token(
    db: impl sqlx::PgExecutor<'_>,
    user_id: Uuid,
    jti: Uuid,
    exp: usize,
) -> Result<(), SqlxError> {
    sqlx::query(
        r#"
    INSERT INTO refresh_tokens (id, user_id, expires_at)
    VALUES ($1, $2, to_timestamp($3))
    "#,
    )
    .bind(jti) // Uuid
    .bind(user_id) // Uuid
    .bind(exp as i64) // the refresh token's exp claim (Unix seconds)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn revoke_refresh_token_if_valid(
    db: impl sqlx::PgExecutor<'_>,
    user_id: Uuid,
    jti: Uuid,
) -> Result<Option<Uuid>, SqlxError> {
    sqlx::query_scalar::<_, Uuid>(
        r#"
    UPDATE refresh_tokens
    SET revoked_at = NOW()
    WHERE id = $1
      AND user_id = $2
      AND revoked_at IS NULL
      AND expires_at > NOW()
    RETURNING id
    "#,
    )
    .bind(jti)
    .bind(user_id)
    .fetch_optional(db)
    .await
}

pub async fn revoke_all_refresh_token(
    db: impl sqlx::PgExecutor<'_>,
    user_id: Uuid,
) -> Result<(), SqlxError> {
    sqlx::query(
        r#"
    UPDATE refresh_tokens
    SET revoked_at = NOW()
    WHERE user_id = $1 AND revoked_at IS NULL
    "#,
    )
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn revoke_refresh_token(
    db: impl sqlx::PgExecutor<'_>,
    user_id: Uuid,
    jti: Uuid,
) -> Result<(), SqlxError> {
    sqlx::query(
        r#"
    UPDATE refresh_tokens
    SET revoked_at = NOW()
    WHERE id = $1
      AND user_id = $2
      AND revoked_at IS NULL
    "#,
    )
    .bind(jti)
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(())
}

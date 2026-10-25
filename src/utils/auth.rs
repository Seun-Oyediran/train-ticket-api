use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Duration;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::auth::{Claims, Token},
    state::AppState,
};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    Ok(Argon2::default()
        .hash_password(password.as_bytes())?
        .to_string())
}

pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    match PasswordHash::new(hashed_password) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

pub fn generate_auth_token(
    sub: Uuid,
    email: String,
    state: &AppState,
) -> Result<(Token, Uuid, usize), AppError> {
    let access_token_claims = Claims::new(
        sub,
        email.clone(),
        Duration::minutes(state.config.access_token_ttl_minutes),
    );

    let refresh_token_claims = Claims::new(
        sub,
        email,
        Duration::days(state.config.refresh_token_ttl_days),
    );

    let token = Claims::generate_response(
        &access_token_claims,
        &refresh_token_claims,
        &state.jwt_encoded_secret,
        &state.jwt_refresh_encoded_secret,
    )?;

    Ok((token, refresh_token_claims.jti, refresh_token_claims.exp))
}

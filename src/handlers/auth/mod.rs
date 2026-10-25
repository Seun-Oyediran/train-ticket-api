use std::{sync::Arc, time::Duration};

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};

use tower_governor::{GovernorError, GovernorLayer, governor::GovernorConfigBuilder};
use validator::Validate;

use crate::{
    error::AppError,
    extractors::AppJson,
    handlers::{
        auth::queries::{
            create_user, find_user_with_password, revoke_all_refresh_token, revoke_refresh_token,
            revoke_refresh_token_if_valid, store_refresh_token,
        },
        not_found::method_not_allowed,
    },
    models::{
        auth::{
            Claims, CreateAccountPayload, LoginPayload, LoginResponse, RefreshTokenPayload, Token,
        },
        user::User,
    },
    state::AppState,
    utils::{
        ResponseBody,
        auth::{generate_auth_token, hash_password, verify_password},
    },
};

mod queries;

pub fn auth_routes() -> Router<AppState> {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(5)
            .burst_size(5)
            .error_handler(|e| {
                let saved_headers = match &e {
                    GovernorError::TooManyRequests { headers, .. } => headers.clone(),
                    _ => None,
                };
                let mut response = AppError::from(e).into_response();

                if let Some(headers) = saved_headers {
                    response.headers_mut().extend(headers);
                }
                response
            })
            .finish()
            .expect("Failed to start rate limiter"),
    );

    let governor_limiter = governor_conf.limiter().clone();
    let interval = Duration::from_secs(60);

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;
            tracing::info!("rate limiting storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });

    Router::new()
        .route(
            "/login",
            post(login).layer(GovernorLayer {
                config: governor_conf,
            }),
        )
        .route("/create", post(create_account))
        .route("/refresh-token", post(refresh_token))
        .route("/logout", post(logout))
        .method_not_allowed_fallback(method_not_allowed)
}

pub async fn login(
    State(state): State<AppState>,
    AppJson(body): AppJson<LoginPayload>,
) -> Result<(StatusCode, Json<ResponseBody<LoginResponse>>), AppError> {
    body.validate()?;

    let user = find_user_with_password(&state.db, body.email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    if !verify_password(&body.password, &user.password) {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }
    let (token, jti, exp) = generate_auth_token(user.user.id, user.user.email.clone(), &state)?;

    store_refresh_token(&state.db, user.user.id, jti, exp).await?;

    let res_data = LoginResponse::new(user.user, token);

    let response = ResponseBody::new(true, String::from("Login successful"), Some(res_data));
    Ok((StatusCode::OK, Json(response)))
}

pub async fn create_account(
    State(state): State<AppState>,
    AppJson(body): AppJson<CreateAccountPayload>,
) -> Result<(StatusCode, Json<ResponseBody<User>>), AppError> {
    body.validate()?;

    let hashed_password = hash_password(&body.password)?;

    let result = create_user(&state.db, body, hashed_password).await?;

    let response = ResponseBody::new(true, String::from("Account created"), Some(result));
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    AppJson(body): AppJson<RefreshTokenPayload>,
) -> Result<(StatusCode, Json<ResponseBody<Token>>), AppError> {
    let claims = Claims::decode_jwt(&body.refresh_token, &state.jwt_refresh_decoded_secret)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let mut tx = state.db.begin().await?;

    let old_jti =
        revoke_refresh_token_if_valid(&mut *tx, claims.claims.sub, claims.claims.jti).await?;

    match old_jti {
        None => {
            revoke_all_refresh_token(&mut *tx, claims.claims.sub).await?;
            tx.commit().await?;
            return Err(AppError::Unauthorized("Unauthorized".to_string()));
        }
        _ => {}
    }

    let (token, jti, exp) = generate_auth_token(claims.claims.sub, claims.claims.email, &state)?;

    store_refresh_token(&mut *tx, claims.claims.sub, jti, exp).await?;
    tx.commit().await?;

    let response = ResponseBody::new(true, String::from("Successful"), Some(token));
    Ok((StatusCode::OK, Json(response)))
}

pub async fn logout(
    State(state): State<AppState>,
    AppJson(body): AppJson<RefreshTokenPayload>,
) -> Result<Json<ResponseBody<()>>, AppError> {
    let claims = Claims::decode_jwt(&body.refresh_token, &state.jwt_refresh_decoded_secret)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    revoke_refresh_token(&state.db, claims.claims.sub, claims.claims.jti).await?;

    Ok(Json(ResponseBody::new(
        true,
        "Logged out".to_string(),
        None,
    )))
}

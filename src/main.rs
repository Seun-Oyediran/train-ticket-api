use std::{net::SocketAddr, time::Duration};

use axum::{Router, http::Method};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;

use crate::{
    config::Config,
    handlers::{app_routes, not_found::not_found},
    state::AppState,
};

mod config;
mod error;
mod extractors;
mod handlers;
mod models;
mod state;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let tracing_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("train_ticket=info,tower_http=info,sqlx=warn"));

    tracing_subscriber::fmt()
        .with_env_filter(tracing_filter)
        .init();

    let config = Config::new().expect("Failed to load config");

    let port = config.port;
    let db_url = &config.database_url;

    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::new(5, 0))
        .connect(db_url)
        .await
        .expect("Failed to connect to db");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate");

    let state = AppState::new(config, pool);

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    let tracing_layer = ServiceBuilder::new().layer(TraceLayer::new_for_http());

    let v1_routes = Router::new().nest("/v1", app_routes());

    let app = Router::new()
        .nest("/api", v1_routes)
        .layer(tracing_layer)
        .with_state(state)
        .fallback(not_found)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("Failed to connect to port");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed to start server");
}

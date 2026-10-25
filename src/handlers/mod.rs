use axum::Router;

use crate::{
    handlers::{
        auth::auth_routes, station_routes::station_routes_routes, tickets::ticket_routes,
        users::user_routes,
    },
    state::AppState,
};

pub mod auth;
pub mod not_found;
pub mod station_routes;
pub mod tickets;
pub mod users;

pub fn app_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_routes())
        .nest("/users", user_routes())
        .nest("/station-routes", station_routes_routes())
        .nest("/tickets", ticket_routes())
}

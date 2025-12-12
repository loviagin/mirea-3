use axum::routing::get;
use axum::Router;

use crate::handlers::AppState;
use crate::handlers::{
    health, iss_trend, last_iss, osdr_list, osdr_sync, space_latest, space_refresh, space_summary,
    trigger_iss,
};

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/last", get(last_iss))
        .route("/fetch", get(trigger_iss))
        .route("/iss/trend", get(iss_trend))
        .route("/osdr/sync", get(osdr_sync))
        .route("/osdr/list", get(osdr_list))
        .route("/space/:src/latest", get(space_latest))
        .route("/space/refresh", get(space_refresh))
        .route("/space/summary", get(space_summary))
}


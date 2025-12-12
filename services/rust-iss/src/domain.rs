use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Domain entities

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssFetchLog {
    pub id: i64,
    pub fetched_at: DateTime<Utc>,
    pub source_url: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsdrItem {
    pub id: i64,
    pub dataset_id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub inserted_at: DateTime<Utc>,
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceCache {
    pub id: i64,
    pub source: String,
    pub fetched_at: DateTime<Utc>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssTrend {
    pub movement: bool,
    pub delta_km: f64,
    pub dt_sec: f64,
    pub velocity_kmh: Option<f64>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub from_lat: Option<f64>,
    pub from_lon: Option<f64>,
    pub to_lat: Option<f64>,
    pub to_lon: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Health {
    pub status: &'static str,
    pub now: DateTime<Utc>,
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found")]
    NotFound,
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::Database(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ),
            ApiError::Http(e) => (
                axum::http::StatusCode::BAD_GATEWAY,
                format!("HTTP error: {}", e),
            ),
            ApiError::Validation(msg) => (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Validation error: {}", msg),
            ),
            ApiError::NotFound => (
                axum::http::StatusCode::NOT_FOUND,
                "Resource not found".to_string(),
            ),
            ApiError::Internal(msg) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                msg,
            ),
        };
        
        let body = serde_json::json!({
            "error": error_message,
        });
        
        (status, axum::Json(body)).into_response()
    }
}


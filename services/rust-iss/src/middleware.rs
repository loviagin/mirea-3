use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use redis::AsyncCommands;

use crate::state::AppState;

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    // Try to get client IP from headers
    let client_ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    // Check rate limit if Redis is available
    if let Some(redis_url) = &state.config.redis_url {
        if let Ok(client) = redis::Client::open(redis_url.as_str()) {
            if let Ok(mut conn) = client.get_async_connection().await {
                let key = format!("rate_limit:{}", client_ip);
                let limit = state.config.rate_limit_per_minute;
                
                let count: u32 = conn
                    .incr(&key, 1)
                    .await
                    .unwrap_or(0);
                
                if count == 1 {
                    let _: () = conn
                        .expire(&key, 60)
                        .await
                        .unwrap_or_default();
                }
                
                if count > limit {
                    return (
                        StatusCode::TOO_MANY_REQUESTS,
                        axum::Json(serde_json::json!({
                            "error": "Rate limit exceeded",
                            "limit": limit,
                            "retry_after": 60,
                        })),
                    )
                        .into_response();
                }
            }
        }
    }
    
    next.run(request).await
}


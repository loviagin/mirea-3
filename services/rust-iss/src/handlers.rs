use std::collections::HashMap;
use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::Value;

use crate::domain::{ApiError, Health};
use crate::services::{IssService, OsdrService, SpaceCacheService};
use crate::validation::Validator;

pub type AppState = crate::state::AppState;

pub async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        now: chrono::Utc::now(),
    })
}

pub async fn last_iss(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let service = IssService::new(state.pool.clone(), state.iss_client.clone());
    match service.get_last().await? {
        Some(log) => Ok(Json(serde_json::json!({
            "id": log.id,
            "fetched_at": log.fetched_at,
            "source_url": log.source_url,
            "payload": log.payload,
        }))),
        None => Ok(Json(serde_json::json!({"message": "no data"}))),
    }
}

pub async fn trigger_iss(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let service = IssService::new(state.pool.clone(), state.iss_client.clone());
    service.fetch_and_store().await?;
    last_iss(State(state)).await
}

pub async fn iss_trend(State(state): State<AppState>) -> Result<Json<crate::domain::IssTrend>, ApiError> {
    let service = IssService::new(state.pool.clone(), state.iss_client.clone());
    Ok(Json(service.calculate_trend().await?))
}

pub async fn osdr_sync(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let service = OsdrService::new(state.pool.clone(), state.nasa_client.clone());
    let written = service.sync().await?;
    Ok(Json(serde_json::json!({ "written": written })))
}

pub async fn osdr_list(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let limit_str = params.get("limit").cloned().unwrap_or_else(|| "20".to_string());
    let limit = limit_str.parse::<i64>()
        .map_err(|_| ApiError::Validation("invalid limit parameter".to_string()))?;
    let limit = Validator::validate_limit(Some(limit))?;
    
    let service = OsdrService::new(state.pool.clone(), state.nasa_client.clone());
    let items = service.list(limit).await?;
    
    let out: Vec<Value> = items.into_iter().map(|item| {
        serde_json::json!({
            "id": item.id,
            "dataset_id": item.dataset_id,
            "title": item.title,
            "status": item.status,
            "updated_at": item.updated_at,
            "inserted_at": item.inserted_at,
            "raw": item.raw,
        })
    }).collect();
    
    Ok(Json(serde_json::json!({ "items": out })))
}

pub async fn space_latest(
    Path(src): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    Validator::validate_source(&src)?;
    
    let service = SpaceCacheService::new(
        state.pool.clone(),
        state.nasa_client.clone(),
        state.spacex_client.clone(),
    );
    
    match service.get_latest(&src).await? {
        Some(cache) => Ok(Json(serde_json::json!({
            "source": cache.source,
            "fetched_at": cache.fetched_at,
            "payload": cache.payload,
        }))),
        None => Ok(Json(serde_json::json!({
            "source": src,
            "message": "no data"
        }))),
    }
}

pub async fn space_refresh(
    Query(q): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let list = q.get("src").cloned().unwrap_or_else(|| "apod,neo,flr,cme,spacex".to_string());
    let mut done = Vec::new();
    
    let service = SpaceCacheService::new(
        state.pool.clone(),
        state.nasa_client.clone(),
        state.spacex_client.clone(),
    );
    
    for s in list.split(',').map(|x| x.trim().to_lowercase()) {
        match s.as_str() {
            "apod" => {
                if let Err(e) = service.fetch_apod().await {
                    tracing::error!("Failed to fetch apod: {:?}", e);
                } else {
                    done.push("apod");
                }
            }
            "neo" => {
                if let Err(e) = service.fetch_neo().await {
                    tracing::error!("Failed to fetch neo: {:?}", e);
                } else {
                    done.push("neo");
                }
            }
            "flr" => {
                if let Err(e) = service.fetch_donki_flr().await {
                    tracing::error!("Failed to fetch flr: {:?}", e);
                } else {
                    done.push("flr");
                }
            }
            "cme" => {
                if let Err(e) = service.fetch_donki_cme().await {
                    tracing::error!("Failed to fetch cme: {:?}", e);
                } else {
                    done.push("cme");
                }
            }
            "spacex" => {
                if let Err(e) = service.fetch_spacex().await {
                    tracing::error!("Failed to fetch spacex: {:?}", e);
                } else {
                    done.push("spacex");
                }
            }
            _ => {}
        }
    }
    
    Ok(Json(serde_json::json!({ "refreshed": done })))
}

pub async fn space_summary(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let service = SpaceCacheService::new(
        state.pool.clone(),
        state.nasa_client.clone(),
        state.spacex_client.clone(),
    );
    
    let apod = service.get_latest("apod").await?
        .map(|c| serde_json::json!({"at": c.fetched_at, "payload": c.payload}))
        .unwrap_or(serde_json::json!({}));
    
    let neo = service.get_latest("neo").await?
        .map(|c| serde_json::json!({"at": c.fetched_at, "payload": c.payload}))
        .unwrap_or(serde_json::json!({}));
    
    let flr = service.get_latest("flr").await?
        .map(|c| serde_json::json!({"at": c.fetched_at, "payload": c.payload}))
        .unwrap_or(serde_json::json!({}));
    
    let cme = service.get_latest("cme").await?
        .map(|c| serde_json::json!({"at": c.fetched_at, "payload": c.payload}))
        .unwrap_or(serde_json::json!({}));
    
    let spacex = service.get_latest("spacex").await?
        .map(|c| serde_json::json!({"at": c.fetched_at, "payload": c.payload}))
        .unwrap_or(serde_json::json!({}));
    
    let iss_service = IssService::new(state.pool.clone(), state.iss_client.clone());
    let iss_last = iss_service.get_last().await?
        .map(|l| serde_json::json!({"at": l.fetched_at, "payload": l.payload}))
        .unwrap_or(serde_json::json!({}));
    
    let osdr_service = OsdrService::new(state.pool.clone(), state.nasa_client.clone());
    let osdr_count = osdr_service.count().await?;
    
    Ok(Json(serde_json::json!({
        "apod": apod,
        "neo": neo,
        "flr": flr,
        "cme": cme,
        "spacex": spacex,
        "iss": iss_last,
        "osdr_count": osdr_count,
    })))
}


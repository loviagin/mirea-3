mod config;
mod domain;
mod repo;
mod clients;
mod services;
mod handlers;
mod routes;
mod state;
mod validation;
mod middleware;

use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use axum::middleware::from_fn_with_state;

use crate::config::Config;
use crate::repo::init_db;
use crate::routes::create_router;
use crate::services::{IssService, OsdrService, SpaceCacheService};
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    dotenvy::dotenv().ok();

    let config = Config::from_env();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    
    init_db(&pool).await?;

    let state = AppState::new(pool.clone(), config.clone())?;

    // Background tasks with advisory locks to prevent overlapping
    let intervals = &config.fetch_intervals;
    
    // OSDR background task
    {
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                // Use PostgreSQL advisory lock to prevent overlapping
                let lock_key: i64 = 1001; // Unique key for OSDR
                if let Ok(Some(_)) = sqlx::query_scalar::<_, bool>(
                    "SELECT pg_try_advisory_lock($1)"
                )
                .bind(lock_key)
                .fetch_optional(&state.pool)
                .await
                {
                    let service = OsdrService::new(state.pool.clone(), state.nasa_client.clone());
                    if let Err(e) = service.sync().await {
                        error!("OSDR sync error: {:?}", e);
                    }
                    let _: Result<_, _> = sqlx::query("SELECT pg_advisory_unlock($1)")
                        .bind(lock_key)
                        .execute(&state.pool)
                        .await;
                }
                tokio::time::sleep(Duration::from_secs(intervals.osdr)).await;
            }
        });
    }
    
    // ISS background task
    {
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                let lock_key: i64 = 1002; // Unique key for ISS
                if let Ok(Some(_)) = sqlx::query_scalar::<_, bool>(
                    "SELECT pg_try_advisory_lock($1)"
                )
                .bind(lock_key)
                .fetch_optional(&state.pool)
                .await
                {
                    let service = IssService::new(state.pool.clone(), state.iss_client.clone());
                    if let Err(e) = service.fetch_and_store().await {
                        error!("ISS fetch error: {:?}", e);
                    }
                    let _: Result<_, _> = sqlx::query("SELECT pg_advisory_unlock($1)")
                        .bind(lock_key)
                        .execute(&state.pool)
                        .await;
                }
                tokio::time::sleep(Duration::from_secs(intervals.iss)).await;
            }
        });
    }
    
    // APOD background task
    {
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                let lock_key: i64 = 1003;
                if let Ok(Some(_)) = sqlx::query_scalar::<_, bool>(
                    "SELECT pg_try_advisory_lock($1)"
                )
                .bind(lock_key)
                .fetch_optional(&state.pool)
                .await
                {
                    let service = SpaceCacheService::new(
                        state.pool.clone(),
                        state.nasa_client.clone(),
                        state.spacex_client.clone(),
                    );
                    if let Err(e) = service.fetch_apod().await {
                        error!("APOD fetch error: {:?}", e);
                    }
                    let _: Result<_, _> = sqlx::query("SELECT pg_advisory_unlock($1)")
                        .bind(lock_key)
                        .execute(&state.pool)
                        .await;
                }
                tokio::time::sleep(Duration::from_secs(intervals.apod)).await;
            }
        });
    }
    
    // NEO background task
    {
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                let lock_key: i64 = 1004;
                if let Ok(Some(_)) = sqlx::query_scalar::<_, bool>(
                    "SELECT pg_try_advisory_lock($1)"
                )
                .bind(lock_key)
                .fetch_optional(&state.pool)
                .await
                {
                    let service = SpaceCacheService::new(
                        state.pool.clone(),
                        state.nasa_client.clone(),
                        state.spacex_client.clone(),
                    );
                    if let Err(e) = service.fetch_neo().await {
                        error!("NEO fetch error: {:?}", e);
                    }
                    let _: Result<_, _> = sqlx::query("SELECT pg_advisory_unlock($1)")
                        .bind(lock_key)
                        .execute(&state.pool)
                        .await;
                }
                tokio::time::sleep(Duration::from_secs(intervals.neo)).await;
            }
        });
    }
    
    // DONKI background task
    {
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                let lock_key: i64 = 1005;
                if let Ok(Some(_)) = sqlx::query_scalar::<_, bool>(
                    "SELECT pg_try_advisory_lock($1)"
                )
                .bind(lock_key)
                .fetch_optional(&state.pool)
                .await
                {
                    let service = SpaceCacheService::new(
                        state.pool.clone(),
                        state.nasa_client.clone(),
                        state.spacex_client.clone(),
                    );
                    let _ = service.fetch_donki_flr().await;
                    let _ = service.fetch_donki_cme().await;
                    let _: Result<_, _> = sqlx::query("SELECT pg_advisory_unlock($1)")
                        .bind(lock_key)
                        .execute(&state.pool)
                        .await;
                }
                tokio::time::sleep(Duration::from_secs(intervals.donki)).await;
            }
        });
    }
    
    // SpaceX background task
    {
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                let lock_key: i64 = 1006;
                if let Ok(Some(_)) = sqlx::query_scalar::<_, bool>(
                    "SELECT pg_try_advisory_lock($1)"
                )
                .bind(lock_key)
                .fetch_optional(&state.pool)
                .await
                {
                    let service = SpaceCacheService::new(
                        state.pool.clone(),
                        state.nasa_client.clone(),
                        state.spacex_client.clone(),
                    );
                    if let Err(e) = service.fetch_spacex().await {
                        error!("SpaceX fetch error: {:?}", e);
                    }
                    let _: Result<_, _> = sqlx::query("SELECT pg_advisory_unlock($1)")
                        .bind(lock_key)
                        .execute(&state.pool)
                        .await;
                }
                tokio::time::sleep(Duration::from_secs(intervals.spacex)).await;
            }
        });
    }

    let app = create_router()
        .layer(from_fn_with_state(
            state.clone(),
            middleware::rate_limit_middleware,
        ))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("rust_iss listening on 0.0.0.0:3000");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

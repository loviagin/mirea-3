use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

use crate::domain::{IssFetchLog, OsdrItem, SpaceCache};

pub struct IssRepo;

impl IssRepo {
    pub async fn create(pool: &PgPool, source_url: &str, payload: Value) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO iss_fetch_log (source_url, payload) VALUES ($1, $2) RETURNING id"
        )
        .bind(source_url)
        .bind(payload)
        .fetch_one(pool)
        .await?;
        
        Ok(row.get("id"))
    }
    
    pub async fn find_last(pool: &PgPool) -> Result<Option<IssFetchLog>, sqlx::Error> {
        let row_opt = sqlx::query(
            "SELECT id, fetched_at, source_url, payload
             FROM iss_fetch_log
             ORDER BY id DESC LIMIT 1"
        )
        .fetch_optional(pool)
        .await?;
        
        if let Some(row) = row_opt {
            Ok(Some(IssFetchLog {
                id: row.get("id"),
                fetched_at: row.get::<DateTime<Utc>, _>("fetched_at"),
                source_url: row.get("source_url"),
                payload: row.try_get("payload").unwrap_or(Value::Null),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn find_last_two(pool: &PgPool) -> Result<Vec<IssFetchLog>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, fetched_at, source_url, payload
             FROM iss_fetch_log
             ORDER BY id DESC LIMIT 2"
        )
        .fetch_all(pool)
        .await?;
        
        Ok(rows.into_iter().map(|row| IssFetchLog {
            id: row.get("id"),
            fetched_at: row.get::<DateTime<Utc>, _>("fetched_at"),
            source_url: row.get("source_url"),
            payload: row.try_get("payload").unwrap_or(Value::Null),
        }).collect())
    }
}

pub struct OsdrRepo;

impl OsdrRepo {
    pub async fn upsert(
        pool: &PgPool,
        dataset_id: Option<&str>,
        title: Option<&str>,
        status: Option<&str>,
        updated_at: Option<DateTime<Utc>>,
        raw: Value,
    ) -> Result<i64, sqlx::Error> {
        if let Some(ds_id) = dataset_id {
            // Upsert by dataset_id
            let row = sqlx::query(
                "INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
                 VALUES($1, $2, $3, $4, $5)
                 ON CONFLICT (dataset_id) DO UPDATE
                 SET title=EXCLUDED.title, 
                     status=EXCLUDED.status,
                     updated_at=EXCLUDED.updated_at, 
                     raw=EXCLUDED.raw
                 RETURNING id"
            )
            .bind(ds_id)
            .bind(title)
            .bind(status)
            .bind(updated_at)
            .bind(raw)
            .fetch_one(pool)
            .await?;
            
            Ok(row.get("id"))
        } else {
            // Insert without dataset_id
            let row = sqlx::query(
                "INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
                 VALUES($1, $2, $3, $4, $5)
                 RETURNING id"
            )
            .bind::<Option<String>>(None)
            .bind(title)
            .bind(status)
            .bind(updated_at)
            .bind(raw)
            .fetch_one(pool)
            .await?;
            
            Ok(row.get("id"))
        }
    }
    
    pub async fn list(pool: &PgPool, limit: i64) -> Result<Vec<OsdrItem>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, dataset_id, title, status, updated_at, inserted_at, raw
             FROM osdr_items
             ORDER BY inserted_at DESC
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
        
        Ok(rows.into_iter().map(|row| OsdrItem {
            id: row.get("id"),
            dataset_id: row.get("dataset_id"),
            title: row.get("title"),
            status: row.get("status"),
            updated_at: row.get("updated_at"),
            inserted_at: row.get::<DateTime<Utc>, _>("inserted_at"),
            raw: row.get("raw"),
        }).collect())
    }
    
    pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT count(*) AS c FROM osdr_items")
            .fetch_one(pool)
            .await?;
        
        Ok(row.get::<i64, _>("c"))
    }
}

pub struct CacheRepo;

impl CacheRepo {
    pub async fn create(pool: &PgPool, source: &str, payload: Value) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO space_cache(source, payload) VALUES ($1, $2) RETURNING id"
        )
        .bind(source)
        .bind(payload)
        .fetch_one(pool)
        .await?;
        
        Ok(row.get("id"))
    }
    
    pub async fn find_latest(pool: &PgPool, source: &str) -> Result<Option<SpaceCache>, sqlx::Error> {
        let row_opt = sqlx::query(
            "SELECT id, source, fetched_at, payload
             FROM space_cache
             WHERE source = $1
             ORDER BY id DESC LIMIT 1"
        )
        .bind(source)
        .fetch_optional(pool)
        .await?;
        
        if let Some(row) = row_opt {
            Ok(Some(SpaceCache {
                id: row.get("id"),
                source: row.get("source"),
                fetched_at: row.get::<DateTime<Utc>, _>("fetched_at"),
                payload: row.get("payload"),
            }))
        } else {
            Ok(None)
        }
    }
}

pub async fn init_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    // ISS table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS iss_fetch_log(
            id BIGSERIAL PRIMARY KEY,
            fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            source_url TEXT NOT NULL,
            payload JSONB NOT NULL
        )"
    )
    .execute(pool)
    .await?;
    
    // OSDR table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS osdr_items(
            id BIGSERIAL PRIMARY KEY,
            dataset_id TEXT,
            title TEXT,
            status TEXT,
            updated_at TIMESTAMPTZ,
            inserted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            raw JSONB NOT NULL
        )"
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS ux_osdr_dataset_id
         ON osdr_items(dataset_id) WHERE dataset_id IS NOT NULL"
    )
    .execute(pool)
    .await?;
    
    // Space cache table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS space_cache(
            id BIGSERIAL PRIMARY KEY,
            source TEXT NOT NULL,
            fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            payload JSONB NOT NULL
        )"
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS ix_space_cache_source 
         ON space_cache(source, fetched_at DESC)"
    )
    .execute(pool)
    .await?;
    
    Ok(())
}


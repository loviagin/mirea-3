use chrono::{DateTime, Days, NaiveDateTime, TimeZone, Utc};
use serde_json::Value;
use sqlx::PgPool;

use std::sync::Arc;
use crate::clients::{IssClient, NasaClient, SpaceXClient};
use crate::domain::ApiError;
use crate::repo::{CacheRepo, IssRepo, OsdrRepo};

pub struct IssService {
    repo: IssRepo,
    client: Arc<IssClient>,
    pool: PgPool,
}

impl IssService {
    pub fn new(pool: PgPool, client: Arc<IssClient>) -> Self {
        Self {
            repo: IssRepo,
            client,
            pool,
        }
    }
    
    pub async fn fetch_and_store(&self) -> Result<(), ApiError> {
        let json = self.client.fetch_current().await?;
        let source_url = self.client.base_url.clone();
        
        IssRepo::create(&self.pool, &source_url, json).await?;
        Ok(())
    }
    
    pub async fn get_last(&self) -> Result<Option<crate::domain::IssFetchLog>, ApiError> {
        Ok(IssRepo::find_last(&self.pool).await?)
    }
    
    pub async fn calculate_trend(&self) -> Result<crate::domain::IssTrend, ApiError> {
        let logs = IssRepo::find_last_two(&self.pool).await?;
        
        if logs.len() < 2 {
            return Ok(crate::domain::IssTrend {
                movement: false,
                delta_km: 0.0,
                dt_sec: 0.0,
                velocity_kmh: None,
                from_time: None,
                to_time: None,
                from_lat: None,
                from_lon: None,
                to_lat: None,
                to_lon: None,
            });
        }
        
        let t2 = logs[0].fetched_at;
        let t1 = logs[1].fetched_at;
        let p2 = &logs[0].payload;
        let p1 = &logs[1].payload;
        
        let lat1 = extract_number(&p1["latitude"]);
        let lon1 = extract_number(&p1["longitude"]);
        let lat2 = extract_number(&p2["latitude"]);
        let lon2 = extract_number(&p2["longitude"]);
        let v2 = extract_number(&p2["velocity"]);
        
        let mut delta_km = 0.0;
        let mut movement = false;
        
        if let (Some(a1), Some(o1), Some(a2), Some(o2)) = (lat1, lon1, lat2, lon2) {
            delta_km = haversine_km(a1, o1, a2, o2);
            movement = delta_km > 0.1;
        }
        
        let dt_sec = (t2 - t1).num_milliseconds() as f64 / 1000.0;
        
        Ok(crate::domain::IssTrend {
            movement,
            delta_km,
            dt_sec,
            velocity_kmh: v2,
            from_time: Some(t1),
            to_time: Some(t2),
            from_lat: lat1,
            from_lon: lon1,
            to_lat: lat2,
            to_lon: lon2,
        })
    }
}

pub struct OsdrService {
    repo: OsdrRepo,
    client: Arc<NasaClient>,
    pool: PgPool,
}

impl OsdrService {
    pub fn new(pool: PgPool, client: Arc<NasaClient>) -> Self {
        Self {
            repo: OsdrRepo,
            client,
            pool,
        }
    }
    
    pub async fn sync(&self) -> Result<usize, ApiError> {
        let json = self.client.fetch_osdr().await?;
        
        let items = extract_array(&json);
        let mut written = 0;
        
        for item in items {
            let dataset_id = extract_string(&item, &["dataset_id", "id", "uuid", "studyId", "accession", "osdr_id"]);
            let title = extract_string(&item, &["title", "name", "label"]);
            let status = extract_string(&item, &["status", "state", "lifecycle"]);
            let updated_at = extract_datetime(&item, &["updated", "updated_at", "modified", "lastUpdated", "timestamp"]);
            
            OsdrRepo::upsert(
                &self.pool,
                dataset_id.as_deref(),
                title.as_deref(),
                status.as_deref(),
                updated_at,
                item,
            ).await?;
            
            written += 1;
        }
        
        Ok(written)
    }
    
    pub async fn list(&self, limit: i64) -> Result<Vec<crate::domain::OsdrItem>, ApiError> {
        Ok(OsdrRepo::list(&self.pool, limit).await?)
    }
    
    pub async fn count(&self) -> Result<i64, ApiError> {
        Ok(OsdrRepo::count(&self.pool).await?)
    }
}

pub struct SpaceCacheService {
    repo: CacheRepo,
    nasa_client: Arc<NasaClient>,
    spacex_client: Arc<SpaceXClient>,
    pool: PgPool,
}

impl SpaceCacheService {
    pub fn new(pool: PgPool, nasa_client: Arc<NasaClient>, spacex_client: Arc<SpaceXClient>) -> Self {
        Self {
            repo: CacheRepo,
            nasa_client,
            spacex_client,
            pool,
        }
    }
    
    pub async fn fetch_apod(&self) -> Result<(), ApiError> {
        let json = self.nasa_client.fetch_apod().await?;
        CacheRepo::create(&self.pool, "apod", json).await?;
        Ok(())
    }
    
    pub async fn fetch_neo(&self) -> Result<(), ApiError> {
        let today = Utc::now().date_naive();
        let start = today - Days::new(2);
        let json = self.nasa_client.fetch_neo_feed(&start.to_string(), &today.to_string()).await?;
        CacheRepo::create(&self.pool, "neo", json).await?;
        Ok(())
    }
    
    pub async fn fetch_donki_flr(&self) -> Result<(), ApiError> {
        let (from, to) = last_days(5);
        let json = self.nasa_client.fetch_donki_flr(&from, &to).await?;
        CacheRepo::create(&self.pool, "flr", json).await?;
        Ok(())
    }
    
    pub async fn fetch_donki_cme(&self) -> Result<(), ApiError> {
        let (from, to) = last_days(5);
        let json = self.nasa_client.fetch_donki_cme(&from, &to).await?;
        CacheRepo::create(&self.pool, "cme", json).await?;
        Ok(())
    }
    
    pub async fn fetch_spacex(&self) -> Result<(), ApiError> {
        let json = self.spacex_client.fetch_next_launch().await?;
        CacheRepo::create(&self.pool, "spacex", json).await?;
        Ok(())
    }
    
    pub async fn get_latest(&self, source: &str) -> Result<Option<crate::domain::SpaceCache>, ApiError> {
        Ok(CacheRepo::find_latest(&self.pool, source).await?)
    }
}

// Helper functions
fn extract_string(v: &Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(x) = v.get(*k) {
            if let Some(s) = x.as_str() {
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            } else if x.is_number() {
                return Some(x.to_string());
            }
        }
    }
    None
}

fn extract_number(v: &Value) -> Option<f64> {
    if let Some(x) = v.as_f64() {
        return Some(x);
    }
    if let Some(s) = v.as_str() {
        return s.parse::<f64>().ok();
    }
    None
}

fn extract_datetime(v: &Value, keys: &[&str]) -> Option<DateTime<Utc>> {
    for k in keys {
        if let Some(x) = v.get(*k) {
            if let Some(s) = x.as_str() {
                if let Ok(dt) = s.parse::<DateTime<Utc>>() {
                    return Some(dt);
                }
                if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                    return Some(Utc.from_utc_datetime(&ndt));
                }
            } else if let Some(n) = x.as_i64() {
                return Some(Utc.timestamp_opt(n, 0).single().unwrap_or_else(Utc::now));
            }
        }
    }
    None
}

fn extract_array(json: &Value) -> Vec<Value> {
    if let Some(a) = json.as_array() {
        a.clone()
    } else if let Some(v) = json.get("items").and_then(|x| x.as_array()) {
        v.clone()
    } else if let Some(v) = json.get("results").and_then(|x| x.as_array()) {
        v.clone()
    } else {
        vec![json.clone()]
    }
}

fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let rlat1 = lat1.to_radians();
    let rlat2 = lat2.to_radians();
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2) + rlat1.cos() * rlat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    6371.0 * c
}

fn last_days(n: i64) -> (String, String) {
    let to = Utc::now().date_naive();
    let from = to - Days::new(n as u64);
    (from.to_string(), to.to_string())
}


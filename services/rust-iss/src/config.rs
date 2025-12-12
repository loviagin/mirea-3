use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub nasa_url: String,
    pub nasa_key: String,
    pub where_iss_url: String,
    pub fetch_intervals: FetchIntervals,
    pub http_timeout: Duration,
    pub http_retries: u32,
    pub rate_limit_per_minute: u32,
}

#[derive(Clone, Debug)]
pub struct FetchIntervals {
    pub osdr: u64,
    pub iss: u64,
    pub apod: u64,
    pub neo: u64,
    pub donki: u64,
    pub spacex: u64,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL is required");
        
        let redis_url = std::env::var("REDIS_URL").ok();
        
        let nasa_url = std::env::var("NASA_API_URL")
            .unwrap_or_else(|_| "https://visualization.osdr.nasa.gov/biodata/api/v2/datasets/?format=json".to_string());
        
        let nasa_key = std::env::var("NASA_API_KEY").unwrap_or_default();
        
        let where_iss_url = std::env::var("WHERE_ISS_URL")
            .unwrap_or_else(|_| "https://api.wheretheiss.at/v1/satellites/25544".to_string());
        
        let fetch_intervals = FetchIntervals {
            osdr: env_u64("FETCH_EVERY_SECONDS", 600),
            iss: env_u64("ISS_EVERY_SECONDS", 120),
            apod: env_u64("APOD_EVERY_SECONDS", 43200),
            neo: env_u64("NEO_EVERY_SECONDS", 7200),
            donki: env_u64("DONKI_EVERY_SECONDS", 3600),
            spacex: env_u64("SPACEX_EVERY_SECONDS", 3600),
        };
        
        let http_timeout = Duration::from_secs(
            env_u64("HTTP_TIMEOUT_SECONDS", 30) as u64
        );
        
        let http_retries = env_u64("HTTP_RETRIES", 3) as u32;
        
        let rate_limit_per_minute = env_u64("RATE_LIMIT_PER_MINUTE", 60) as u32;
        
        Self {
            database_url,
            redis_url,
            nasa_url,
            nasa_key,
            where_iss_url,
            fetch_intervals,
            http_timeout,
            http_retries,
            rate_limit_per_minute,
        }
    }
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}


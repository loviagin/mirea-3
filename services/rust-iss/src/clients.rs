use std::time::Duration;
use reqwest::Client;
use serde_json::Value;
use tracing::warn;

use crate::config::Config;
use crate::domain::ApiError;

pub struct HttpClient {
    client: Client,
    config: Config,
}

impl HttpClient {
    pub fn new(config: Config) -> Result<Self, ApiError> {
        let client = Client::builder()
            .timeout(config.http_timeout)
            .user_agent("rust_iss/1.0")
            .build()?;
        
        Ok(Self { client, config })
    }
    
    pub async fn get_with_retry(&self, url: &str) -> Result<Value, ApiError> {
        let mut last_error = None;
        
        for attempt in 0..=self.config.http_retries {
            match self.client.get(url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return resp.json().await.map_err(ApiError::Http);
                    } else {
                        warn!("HTTP {} for {} (attempt {})", resp.status(), url, attempt + 1);
                        if attempt < self.config.http_retries {
                            tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                            continue;
                        }
                        return Err(ApiError::Internal(format!("HTTP {} from {}", resp.status(), url)));
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.http_retries {
                        warn!("Request failed (attempt {}): {:?}", attempt + 1, last_error);
                        tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    }
                }
            }
        }
        
        Err(ApiError::Http(last_error.unwrap()))
    }
}

pub struct IssClient {
    http: HttpClient,
    pub base_url: String,
}

impl IssClient {
    pub fn new(config: Config) -> Result<Self, ApiError> {
        Ok(Self {
            http: HttpClient::new(config.clone())?,
            base_url: config.where_iss_url,
        })
    }
    
    pub async fn fetch_current(&self) -> Result<Value, ApiError> {
        self.http.get_with_retry(&self.base_url).await
    }
}

pub struct NasaClient {
    http: HttpClient,
    base_url: String,
    api_key: String,
}

impl NasaClient {
    pub fn new(config: Config) -> Result<Self, ApiError> {
        Ok(Self {
            http: HttpClient::new(config.clone())?,
            base_url: config.nasa_url,
            api_key: config.nasa_key,
        })
    }
    
    pub async fn fetch_osdr(&self) -> Result<Value, ApiError> {
        self.http.get_with_retry(&self.base_url).await
    }
    
    pub async fn fetch_apod(&self) -> Result<Value, ApiError> {
        let api_key_param = if self.api_key.is_empty() {
            String::new()
        } else {
            format!("&api_key={}", self.api_key)
        };
        let url = format!("https://api.nasa.gov/planetary/apod?thumbs=true{}", api_key_param);
        self.http.get_with_retry(&url).await
    }
    
    pub async fn fetch_neo_feed(&self, start_date: &str, end_date: &str) -> Result<Value, ApiError> {
        let api_key_param = if self.api_key.is_empty() {
            String::new()
        } else {
            format!("&api_key={}", self.api_key)
        };
        let url = format!("https://api.nasa.gov/neo/rest/v1/feed?start_date={}&end_date={}{}",
            start_date, end_date, api_key_param);
        self.http.get_with_retry(&url).await
    }
    
    pub async fn fetch_donki_flr(&self, start_date: &str, end_date: &str) -> Result<Value, ApiError> {
        let api_key_param = if self.api_key.is_empty() {
            String::new()
        } else {
            format!("&api_key={}", self.api_key)
        };
        let url = format!("https://api.nasa.gov/DONKI/FLR?startDate={}&endDate={}{}",
            start_date, end_date, api_key_param);
        self.http.get_with_retry(&url).await
    }
    
    pub async fn fetch_donki_cme(&self, start_date: &str, end_date: &str) -> Result<Value, ApiError> {
        let api_key_param = if self.api_key.is_empty() {
            String::new()
        } else {
            format!("&api_key={}", self.api_key)
        };
        let url = format!("https://api.nasa.gov/DONKI/CME?startDate={}&endDate={}{}",
            start_date, end_date, api_key_param);
        self.http.get_with_retry(&url).await
    }
}

pub struct SpaceXClient {
    http: HttpClient,
}

impl SpaceXClient {
    pub fn new(config: Config) -> Result<Self, ApiError> {
        Ok(Self {
            http: HttpClient::new(config)?,
        })
    }
    
    pub async fn fetch_next_launch(&self) -> Result<Value, ApiError> {
        self.http.get_with_retry("https://api.spacexdata.com/v4/launches/next").await
    }
}


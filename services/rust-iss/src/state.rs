use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

use crate::clients::{IssClient, NasaClient, SpaceXClient};
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub iss_client: Arc<IssClient>,
    pub nasa_client: Arc<NasaClient>,
    pub spacex_client: Arc<SpaceXClient>,
    pub config: Config,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Result<Self, crate::domain::ApiError> {
        Ok(Self {
            pool,
            iss_client: Arc::new(IssClient::new(config.clone())?),
            nasa_client: Arc::new(NasaClient::new(config.clone())?),
            spacex_client: Arc::new(SpaceXClient::new(config.clone())?),
            config,
        })
    }
}


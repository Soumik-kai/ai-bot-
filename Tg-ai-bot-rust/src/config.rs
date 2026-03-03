use crate::adapters::{search::SearchAdapterImpl, llm_pool::LLMPool, image::ImageAdapterImpl};
use reqwest::Client;
use sqlx::PgPool;
use redis::Client as RedisClient;
use std::sync::Arc;

#[derive(Clone)]
pub struct Config {
    pub telegram_token: String,
    pub telegram_bot_username: String,
    pub allowed_group_id: i64,
    pub database_dsn: String,
    pub redis_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            telegram_token: std::env::var("TELEGRAM_TOKEN")?,
            telegram_bot_username: std::env::var("TELEGRAM_BOT_USERNAME").unwrap_or_default(),
            allowed_group_id: std::env::var("TELEGRAM_GROUP_ID").unwrap_or_else(|_| "0".into()).parse().unwrap_or(0),
            database_dsn: std::env::var("DATABASE_DSN")?,
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".into()),
        })
    }
}

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub db: PgPool,
    pub redis: RedisClient,
    pub http: Client,
    pub search: Arc<SearchAdapterImpl>,
    pub llm_pool: Arc<LLMPool>,
    pub image: Arc<ImageAdapterImpl>,
}

impl AppState {
    pub async fn new(cfg: &Config) -> anyhow::Result<Self> {
        let db = PgPool::connect(&cfg.database_dsn).await?;
        let redis = RedisClient::open(cfg.redis_url.clone())?;
        let http = Client::new();

        let search = Arc::new(SearchAdapterImpl::new(http.clone()).await?);
        let llm_pool = Arc::new(LLMPool::new(http.clone()).await?);
        let image = Arc::new(ImageAdapterImpl::new(http.clone()).await?);

        Ok(Self {
            cfg: cfg.clone(),
            db,
            redis,
            http,
            search,
            llm_pool,
            image,
        })
    }
}
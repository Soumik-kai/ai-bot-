use axum::{routing::post, Router, Json};
use std::net::SocketAddr;
use tracing_subscriber;
mod config;
mod db;
mod handlers;
mod adapters;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let cfg = config::Config::from_env()?;
    let state = config::AppState::new(&cfg).await?;

    use axum::{routing::{get, post}, Router};

let app = Router::new()
    // Root route for health check
    .route("/", get(|| async { "Bot is running ✅" }))
    // Webhook route for Telegram
    .route("/webhook", post(handlers::webhook_handler))
    .with_state(state);

    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "3000".into()).parse().unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

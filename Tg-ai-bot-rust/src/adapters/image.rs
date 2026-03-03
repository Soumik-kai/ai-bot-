use async_trait::async_trait;
use reqwest::Client;
use anyhow::Result;

/// Image adapter trait
#[async_trait]
pub trait ImageAdapter: Send + Sync {
    async fn generate_image(&self, prompt: &str) -> Result<Vec<u8>>;
}

/// Implementation skeleton for Stable Horde / Hugging Face fallback
pub struct ImageAdapterImpl {
    http: Client,
    // keys and rotation state
}

impl ImageAdapterImpl {
    pub async fn new(http: Client) -> Result<Self> {
        Ok(Self { http })
    }
}

#[async_trait]
impl ImageAdapter for ImageAdapterImpl {
    async fn generate_image(&self, prompt: &str) -> Result<Vec<u8>> {
        // TODO: call Stable Horde or HF inference endpoints with key rotation.
        // Placeholder: return empty bytes (caller should handle).
        tracing::info!("generate_image called with prompt: {}", prompt);
        Ok(vec![])
    }
}
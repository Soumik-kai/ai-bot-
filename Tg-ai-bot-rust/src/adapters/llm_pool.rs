use async_trait::async_trait;
use reqwest::Client;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn call(&self, prompt: &str, max_tokens: usize) -> Result<String>;
    async fn name(&self) -> String;
    async fn is_available(&self) -> bool;
}

/// Pool that tries providers in priority order and falls back on errors.
pub struct LLMPool {
    providers: Vec<Arc<dyn LLMProvider>>,
}

impl LLMPool {
    pub async fn new(_http: Client) -> Result<Self> {
        // TODO: instantiate real providers (OpenRouter GLM-5, Kimi, MiniMax, etc.)
        // For now, create a dummy provider to keep the flow working.
        Ok(Self { providers: vec![Arc::new(DummyProvider::new())] })
    }

    pub async fn call_with_fallback(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        for p in &self.providers {
            if !p.is_available().await {
                continue;
            }
            match p.call(prompt, max_tokens).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    tracing::error!("Provider {} failed: {:?}", p.name().await, e);
                    continue;
                }
            }
        }
        Err(anyhow::anyhow!("All LLM providers failed"))
    }
}

/// Dummy provider for skeleton
struct DummyProvider {
    available: RwLock<bool>,
}

impl DummyProvider {
    fn new() -> Self {
        Self { available: RwLock::new(true) }
    }
}

#[async_trait]
impl LLMProvider for DummyProvider {
    async fn call(&self, prompt: &str, _max_tokens: usize) -> Result<String> {
        Ok(format!("(dummy LLM) Answer based on prompt: {}", &prompt.chars().take(200).collect::<String>()))
    }
    async fn name(&self) -> String {
        "dummy-provider".into()
    }
    async fn is_available(&self) -> bool {
        *self.available.read().await
    }
}
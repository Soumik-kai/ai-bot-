use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
}

#[async_trait]
pub trait SearchAdapter: Send + Sync {
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>>;
}

pub struct SearchAdapterImpl {
    http: Client,
}

impl SearchAdapterImpl {
    pub async fn new(http: Client) -> Result<Self> {
        Ok(Self { http })
    }
}

#[derive(Debug, Deserialize)]
struct DuckDuckGoResponse {
    #[serde(rename = "AbstractText")]
    abstract_text: String,
    #[serde(rename = "AbstractURL")]
    abstract_url: String,
    #[serde(rename = "Heading")]
    heading: String,
    #[serde(rename = "RelatedTopics")]
    related: Option<Vec<RelatedTopic>>,
}

#[derive(Debug, Deserialize)]
struct RelatedTopic {
    #[serde(rename = "Text")]
    text: String,
    #[serde(rename = "FirstURL")]
    url: String,
}

#[async_trait]
impl SearchAdapter for SearchAdapterImpl {
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let url = format!("https://api.duckduckgo.com/?q={}&format=json", query);
        let resp = self.http.get(&url).send().await?.json::<DuckDuckGoResponse>().await?;

        let mut results = Vec::new();

        if !resp.abstract_text.is_empty() {
            results.push(SearchResult {
                title: resp.heading.clone(),
                snippet: resp.abstract_text.clone(),
                url: resp.abstract_url.clone(),
            });
        }

        if let Some(related) = resp.related {
            for r in related.iter().take(5) {
                results.push(SearchResult {
                    title: r.text.clone(),
                    snippet: r.text.clone(),
                    url: r.url.clone(),
                });
            }
        }

        Ok(results)
    }
}

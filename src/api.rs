use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;

use crate::models::{ApiCollection, ApiItemsResponse, ApiListResponse, ApiRaindrop};

const BASE_URL: &str = "https://api.raindrop.io/rest/v1";
const PER_PAGE: u32 = 50;
/// 1 request/sec = 60 req/min, half the posted 120 req/min limit
const REQUEST_INTERVAL: Duration = Duration::from_secs(1);

pub struct RaindropClient {
    client: Client,
    token: String,
}

impl RaindropClient {
    pub fn new(token: &str) -> Self {
        RaindropClient {
            client: Client::new(),
            token: token.to_string(),
        }
    }

    pub async fn get_all_raindrops(&self) -> Result<Vec<ApiRaindrop>> {
        let mut all = Vec::new();
        let mut page = 0u32;

        loop {
            let url = format!("{BASE_URL}/raindrops/0");
            let resp: ApiListResponse<ApiRaindrop> = self
                .client
                .get(&url)
                .bearer_auth(&self.token)
                .query(&[
                    ("perpage", PER_PAGE.to_string()),
                    ("page", page.to_string()),
                ])
                .send()
                .await
                .context("failed to fetch raindrops")?
                .error_for_status()
                .context("raindrop API returned error")?
                .json()
                .await
                .context("failed to parse raindrops response")?;

            let count = resp.items.len();
            all.extend(resp.items);

            if count < PER_PAGE as usize {
                break;
            }
            page += 1;
            tokio::time::sleep(REQUEST_INTERVAL).await;
        }

        Ok(all)
    }

    pub async fn get_collections(&self) -> Result<Vec<ApiCollection>> {
        let (root, children) = tokio::try_join!(
            self.fetch_collections("/collections"),
            self.fetch_collections("/collections/childrens"),
        )?;

        let mut all = root;
        all.extend(children);
        Ok(all)
    }

    async fn fetch_collections(&self, path: &str) -> Result<Vec<ApiCollection>> {
        let url = format!("{BASE_URL}{path}");
        let resp: ApiItemsResponse<ApiCollection> = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .with_context(|| format!("failed to fetch {path}"))?
            .error_for_status()
            .with_context(|| format!("API error for {path}"))?
            .json()
            .await
            .with_context(|| format!("failed to parse {path} response"))?;

        Ok(resp.items)
    }
}

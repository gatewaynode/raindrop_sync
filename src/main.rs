mod api;
mod config;
mod models;
mod sync;

use anyhow::{Context, Result};

use api::RaindropClient;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let token = std::env::var("RAINDROP_TOKEN")
        .context("RAINDROP_TOKEN env var not set — get a test token from https://app.raindrop.io/settings/integrations")?;

    let config = Config::load()?;
    let client = RaindropClient::new(&token);

    let count = sync::sync(&client, &config.output_path).await?;
    println!("Synced {count} bookmarks to {}", config.output_path.display());

    Ok(())
}

mod api;
mod config;
mod filter;
mod logging;
mod models;
mod sync;

use anyhow::Result;
use tracing::{error, info};

use api::RaindropClient;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let log_dir = logging::state_dir();
    let _guard = logging::init(&log_dir)?;

    info!("raindrop_sync starting");

    let config = Config::load().inspect_err(|e| error!("Failed to load config: {e:#}"))?;
    let client = RaindropClient::new(&config.api_key);

    let result = sync::sync(&client, &config.output_path)
        .await
        .inspect_err(|e| error!("Sync failed: {e:#}"))?;

    println!("Synced {} bookmarks to {}", result.total, config.output_path.display());
    println!(
        "Filtered views written to {}:",
        config.output_path.parent().unwrap_or(&config.output_path).display()
    );
    println!("  last_day_bookmarks.json   — {} bookmarks", result.filtered.day);
    println!("  last_week_bookmarks.json  — {} bookmarks", result.filtered.week);
    println!("  last_month_bookmarks.json — {} bookmarks", result.filtered.month);

    info!(
        total = result.total,
        day = result.filtered.day,
        week = result.filtered.week,
        month = result.filtered.month,
        "sync complete"
    );

    Ok(())
}

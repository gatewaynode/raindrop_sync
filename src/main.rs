mod api;
mod config;
mod filter;
mod models;
mod sync;

use anyhow::Result;

use api::RaindropClient;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;
    let client = RaindropClient::new(&config.api_key);

    let result = sync::sync(&client, &config.output_path).await?;

    println!("Synced {} bookmarks to {}", result.total, config.output_path.display());
    println!(
        "Filtered views written to {}:",
        config.output_path.parent().unwrap_or(&config.output_path).display()
    );
    println!("  last_day_bookmarks.json   — {} bookmarks", result.filtered.day);
    println!("  last_week_bookmarks.json  — {} bookmarks", result.filtered.week);
    println!("  last_month_bookmarks.json — {} bookmarks", result.filtered.month);

    Ok(())
}

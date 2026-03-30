use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const LOG_PREFIX: &str = "raindrop_sync.log";
const LOG_RETENTION_DAYS: u64 = 7;

/// Returns the XDG state directory for this app:
/// `$XDG_STATE_HOME/raindrop_sync` or `~/.local/state/raindrop_sync`.
pub fn state_dir() -> PathBuf {
    state_dir_from(
        std::env::var("XDG_STATE_HOME").ok(),
        std::env::var("HOME").ok(),
    )
}

/// Initialises file + stderr logging. The returned [`WorkerGuard`] must be
/// kept alive until the end of `main` to ensure buffered log lines are flushed.
pub fn init(log_dir: &Path) -> Result<WorkerGuard> {
    std::fs::create_dir_all(log_dir)
        .with_context(|| format!("failed to create log directory {}", log_dir.display()))?;

    // Best-effort cleanup — log a warning but don't fail the whole run.
    if let Err(e) = cleanup_old_logs(log_dir) {
        eprintln!("Warning: failed to clean up old logs: {e}");
    }

    let file_appender = tracing_appender::rolling::daily(log_dir, LOG_PREFIX);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            // File layer: no ANSI codes, include target module for context.
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .with(
            // Stderr layer: human-friendly, no module path noise.
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(false),
        )
        .init();

    Ok(guard)
}

/// Deletes rotated log files older than [`LOG_RETENTION_DAYS`] days.
fn cleanup_old_logs(dir: &Path) -> Result<()> {
    let cutoff = std::time::SystemTime::now()
        - std::time::Duration::from_secs(LOG_RETENTION_DAYS * 24 * 60 * 60);

    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("failed to read log directory {}", dir.display()))?
    {
        let entry = entry.context("failed to read log directory entry")?;
        let path = entry.path();

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        // tracing-appender produces: "raindrop_sync.log.YYYY-MM-DD"
        // Skip the base name and any file that doesn't match the pattern.
        if !name.starts_with(LOG_PREFIX) || name == LOG_PREFIX {
            continue;
        }

        let modified = match entry.metadata().and_then(|m| m.modified()) {
            Ok(t) => t,
            Err(_) => continue, // can't determine age; leave it
        };

        if modified < cutoff {
            std::fs::remove_file(&path)
                .with_context(|| format!("failed to remove old log file {}", path.display()))?;
        }
    }

    Ok(())
}

fn state_dir_from(xdg_state_home: Option<String>, home: Option<String>) -> PathBuf {
    let base = xdg_state_home
        .map(PathBuf::from)
        .or_else(|| home.map(|h| PathBuf::from(h).join(".local/state")))
        .unwrap_or_else(|| PathBuf::from(".local/state"));
    base.join("raindrop_sync")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_dir_uses_xdg_state_home_when_set() {
        let path = state_dir_from(Some("/custom/state".to_string()), None);
        assert_eq!(path, PathBuf::from("/custom/state/raindrop_sync"));
    }

    #[test]
    fn test_state_dir_defaults_to_home_local_state() {
        let path = state_dir_from(None, Some("/home/user".to_string()));
        assert_eq!(path, PathBuf::from("/home/user/.local/state/raindrop_sync"));
    }
}

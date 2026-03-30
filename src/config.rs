use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

const DEFAULT_OUTPUT_PATH: &str =
    "~/Documents/Claude/Projects/Continual Study and Research/bookmarks.json";

const DEFAULT_CONFIG: &str = r#"# Raindrop.io API key.
# Get your test token from: https://app.raindrop.io/settings/integrations
# The RAINDROP_TOKEN environment variable takes precedence if set.
api_key = ""

# Path where bookmarks.json and the filtered views will be written.
output_path = "~/Documents/Claude/Projects/Continual Study and Research/bookmarks.json"
"#;

#[derive(Deserialize)]
struct RawConfig {
    api_key: Option<String>,
    output_path: Option<String>,
}

pub struct Config {
    pub api_key: String,
    pub output_path: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = config_file_path();
        ensure_config_exists(&config_path)?;

        let text = std::fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;
        let raw = Self::parse_toml(&text)?;

        let env_token = std::env::var("RAINDROP_TOKEN").ok().filter(|s| !s.is_empty());
        let api_key = resolve_api_key(env_token, raw.api_key, &config_path)?;
        let output_path = expand_tilde(
            raw.output_path
                .as_deref()
                .unwrap_or(DEFAULT_OUTPUT_PATH),
        );

        Ok(Config { api_key, output_path })
    }

    fn parse_toml(text: &str) -> Result<RawConfig> {
        toml::from_str(text).context("failed to parse config")
    }
}

/// Resolves the API key: env var takes precedence over config file.
fn resolve_api_key(env_val: Option<String>, config_val: Option<String>, config_path: &Path) -> Result<String> {
    env_val
        .or_else(|| config_val.filter(|s| !s.is_empty()))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "API key not configured.\n\
                 Edit {} and set api_key, or set the RAINDROP_TOKEN environment variable.\n\
                 Get a token from: https://app.raindrop.io/settings/integrations",
                config_path.display()
            )
        })
}

/// Returns `$XDG_CONFIG_HOME/raindrop_sync/config.toml`, falling back to
/// `~/.config/raindrop_sync/config.toml` when `XDG_CONFIG_HOME` is not set.
fn config_file_path() -> PathBuf {
    config_file_path_from(
        std::env::var("XDG_CONFIG_HOME").ok(),
        std::env::var("HOME").ok(),
    )
}

fn config_file_path_from(xdg_config_home: Option<String>, home: Option<String>) -> PathBuf {
    let base = xdg_config_home
        .map(PathBuf::from)
        .or_else(|| home.map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"));
    base.join("raindrop_sync").join("config.toml")
}

/// Creates the config directory and a default config file if neither exist.
fn ensure_config_exists(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    let dir = path.parent().expect("config path has no parent");
    std::fs::create_dir_all(dir)
        .with_context(|| format!("failed to create {}", dir.display()))?;
    std::fs::write(path, DEFAULT_CONFIG)
        .with_context(|| format!("failed to write default config to {}", path.display()))?;
    eprintln!("Created default config at {}", path.display());
    Ok(())
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde_replaces_home() {
        let home = std::env::var("HOME").expect("HOME must be set");
        let result = expand_tilde("~/Documents/foo.json");
        assert_eq!(result, PathBuf::from(&home).join("Documents/foo.json"));
    }

    #[test]
    fn test_expand_tilde_leaves_absolute_path_unchanged() {
        let result = expand_tilde("/absolute/path/file.json");
        assert_eq!(result, PathBuf::from("/absolute/path/file.json"));
    }

    #[test]
    fn test_expand_tilde_leaves_relative_path_unchanged() {
        let result = expand_tilde("relative/path.json");
        assert_eq!(result, PathBuf::from("relative/path.json"));
    }

    #[test]
    fn test_parse_toml_reads_both_fields() {
        let toml = r#"
            api_key = "abc123"
            output_path = "/tmp/bookmarks.json"
        "#;
        let raw = Config::parse_toml(toml).unwrap();
        assert_eq!(raw.api_key.unwrap(), "abc123");
        assert_eq!(raw.output_path.unwrap(), "/tmp/bookmarks.json");
    }

    #[test]
    fn test_parse_toml_empty_gives_nones() {
        let raw = Config::parse_toml("").unwrap();
        assert!(raw.api_key.is_none());
        assert!(raw.output_path.is_none());
    }

    #[test]
    fn test_resolve_api_key_prefers_env_over_config() {
        let result = resolve_api_key(
            Some("env_token".to_string()),
            Some("config_token".to_string()),
            Path::new("config.toml"),
        )
        .unwrap();
        assert_eq!(result, "env_token");
    }

    #[test]
    fn test_resolve_api_key_falls_back_to_config() {
        let result = resolve_api_key(
            None,
            Some("config_token".to_string()),
            Path::new("config.toml"),
        )
        .unwrap();
        assert_eq!(result, "config_token");
    }

    #[test]
    fn test_resolve_api_key_errors_when_config_empty_and_no_env() {
        let result = resolve_api_key(None, Some(String::new()), Path::new("config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_api_key_errors_when_both_absent() {
        let result = resolve_api_key(None, None, Path::new("config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_file_path_uses_xdg_config_home_when_set() {
        let path = config_file_path_from(Some("/custom/config".to_string()), None);
        assert_eq!(
            path,
            PathBuf::from("/custom/config/raindrop_sync/config.toml")
        );
    }

    #[test]
    fn test_config_file_path_defaults_to_home_dotconfig() {
        let path = config_file_path_from(None, Some("/home/user".to_string()));
        assert_eq!(
            path,
            PathBuf::from("/home/user/.config/raindrop_sync/config.toml")
        );
    }
}

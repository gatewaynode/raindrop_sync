use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

const CONFIG_FILE: &str = "config.toml";
const DEFAULT_OUTPUT_DIR: &str =
    "~/Documents/Claude/Projects/Continual Study and Research";

#[derive(Deserialize)]
struct RawConfig {
    output_path: Option<String>,
}

pub struct Config {
    pub output_path: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self> {
        let raw: RawConfig = if std::path::Path::new(CONFIG_FILE).exists() {
            let text = std::fs::read_to_string(CONFIG_FILE)
                .with_context(|| format!("failed to read {CONFIG_FILE}"))?;
            Self::parse_toml(&text)?
        } else {
            RawConfig { output_path: None }
        };

        Ok(Self::from_raw(raw))
    }

    fn parse_toml(text: &str) -> Result<RawConfig> {
        toml::from_str(text).context("failed to parse config.toml")
    }

    fn from_raw(raw: RawConfig) -> Self {
        let path_str = raw
            .output_path
            .unwrap_or_else(|| format!("{DEFAULT_OUTPUT_DIR}/bookmarks.json"));

        Config {
            output_path: expand_tilde(&path_str),
        }
    }
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
    fn test_parse_toml_reads_output_path() {
        let toml = r#"output_path = "/tmp/bookmarks.json""#;
        let raw = Config::parse_toml(toml).unwrap();
        assert_eq!(raw.output_path.unwrap(), "/tmp/bookmarks.json");
    }

    #[test]
    fn test_parse_toml_empty_gives_none() {
        let raw = Config::parse_toml("").unwrap();
        assert!(raw.output_path.is_none());
    }

    #[test]
    fn test_from_raw_uses_default_when_no_path() {
        let config = Config::from_raw(RawConfig { output_path: None });
        let path_str = config.output_path.to_string_lossy();
        assert!(path_str.ends_with("bookmarks.json"));
        assert!(path_str.contains("Continual Study and Research"));
    }

    #[test]
    fn test_from_raw_uses_provided_path() {
        let config = Config::from_raw(RawConfig {
            output_path: Some("/tmp/my_bookmarks.json".to_string()),
        });
        assert_eq!(config.output_path, PathBuf::from("/tmp/my_bookmarks.json"));
    }
}

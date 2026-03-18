use crate::types::Language;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

pub const DEFAULT_INCLUDE_PATTERN: &str =
    r"\.(rs|toml|md|txt|py|js|ts|go|java|cpp|c|h|cs|php|rb|swift|kt)$";
pub const DEFAULT_EXCLUDE_PATTERN: &str =
    r"(target|\.git|node_modules|dist|build|\.vscode|\.idea|__pycache__|\.venv|env)";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Backend {
    pub name: String,
    pub api_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub timeout_secs: Option<u64>,
    pub options: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Llm,
    Pattern,
}

impl std::str::FromStr for Mode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "llm" => Ok(Mode::Llm),
            "pattern" => Ok(Mode::Pattern),
            _ => Err(anyhow::anyhow!("invalid mode: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub default_mode: Mode,
    pub backends: Vec<Backend>,
    pub include_pattern: String,
    pub exclude_pattern: String,
    pub cache_enabled: bool,
    pub cache_ttl_days: Option<u64>,
    pub language: Language,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_mode: Mode::Pattern,
            backends: vec![Backend {
                name: "ollama".into(),
                api_url: "http://localhost:11434/v1/chat/completions".into(),
                api_key: None,
                model: "qwen2.5:0.5b".into(),
                timeout_secs: Some(30),
                options: {
                    let mut map = serde_json::Map::new();
                    map.insert("temperature".into(), serde_json::Value::from(0.2));
                    Some(map)
                },
            }],
            include_pattern: DEFAULT_INCLUDE_PATTERN.into(),
            exclude_pattern: DEFAULT_EXCLUDE_PATTERN.into(),
            cache_enabled: false,
            cache_ttl_days: None,
            language: Language::Ru,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("cannot find config directory"))?
            .join("ai-dir");
        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            fs::create_dir_all(&config_dir)?;
            let default = Config::default();
            let toml = toml::to_string_pretty(&default)?;
            fs::write(&config_path, toml)?;
            eprintln!("Created default config at {}", config_path.display());
            Ok(default)
        } else {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        }
    }

    pub fn merge(&self, local: Config) -> Config {
        Config {
            default_mode: local.default_mode,
            backends: if local.backends.is_empty() {
                self.backends.clone()
            } else {
                local.backends
            },
            include_pattern: local.include_pattern,
            exclude_pattern: local.exclude_pattern,
            cache_enabled: local.cache_enabled,
            cache_ttl_days: local.cache_ttl_days.or(self.cache_ttl_days),
            language: local.language,
        }
    }
}

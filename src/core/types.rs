use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub relative: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description {
    pub path: PathBuf,
    pub relative: String,
    pub text: String,
    pub error: Option<String>,
    pub from_cache: bool,
    pub total_lines: Option<usize>,
    pub long_functions: Vec<(String, usize)>, // устарело, но оставляем для обратной совместимости кэша
    pub functions: Vec<(String, usize)>,      // все функции с размерами
    pub symbols: Vec<String>,
}

impl Description {
    pub fn new(
        file: &FileEntry,
        text: String,
        symbols: Vec<String>,
        functions: Vec<(String, usize)>,
    ) -> Self {
        Self {
            path: file.path.clone(),
            relative: file.relative.clone(),
            text,
            error: None,
            from_cache: false,
            total_lines: None,
            long_functions: vec![],
            functions,
            symbols,
        }
    }

    pub fn cached(
        file: &FileEntry,
        text: String,
        total_lines: Option<usize>,
        long_functions: Vec<(String, usize)>, // устарело, загружается из старого кэша
        functions: Vec<(String, usize)>,
        symbols: Vec<String>,
    ) -> Self {
        Self {
            path: file.path.clone(),
            relative: file.relative.clone(),
            text,
            error: None,
            from_cache: true,
            total_lines,
            long_functions,
            functions,
            symbols,
        }
    }

    pub fn error(file: &FileEntry, err: String) -> Self {
        Self {
            path: file.path.clone(),
            relative: file.relative.clone(),
            text: "[ERROR]".into(),
            error: Some(err),
            from_cache: false,
            total_lines: None,
            long_functions: Vec::new(),
            functions: Vec::new(),
            symbols: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Plain,
    Color,
    Json,
    Xml,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Ru,
}

impl Language {
    pub fn prompt_suffix(&self) -> &'static str {
        match self {
            Language::En => "in English",
            Language::Ru => "по-русски",
        }
    }
}

impl FromStr for Language {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Ok(Language::En),
            "ru" | "russian" => Ok(Language::Ru),
            _ => Err(anyhow::anyhow!("invalid language: {}", s)),
        }
    }
}

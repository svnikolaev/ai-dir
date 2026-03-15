mod config;
mod scanner;
mod analyzer;
mod cache;
mod render;
mod types;

use clap::Parser;
use anyhow::Result;
use std::path::PathBuf;
use crate::config::{Config, Mode};
use crate::types::{OutputFormat, Language};

#[derive(Parser)]
#[command(name = "ai-dir", about = "Generate one-line descriptions for files in a directory", version)]
struct Args {
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short, long)]
    mode: Option<String>,
    #[arg(short, long)]
    depth: Option<usize>,
    #[arg(long, value_enum, default_value_t = OutputFormat::Color)]
    format: OutputFormat,
    #[arg(long)]
    include: Option<String>,
    #[arg(long)]
    exclude: Option<String>,
    #[arg(long)]
    no_cache: bool,
    #[arg(long, value_enum)]
    lang: Option<Language>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let global_config = Config::load()?;

    let local_config_path = args.path.join(".ai-dir.toml");
    let local_config = if local_config_path.exists() {
        let content = std::fs::read_to_string(local_config_path)?;
        Some(toml::from_str::<Config>(&content)?)
    } else {
        None
    };

    let mut config = if let Some(local) = local_config {
        global_config.merge(local)
    } else {
        global_config
    };

    if let Some(mode_str) = args.mode {
        config.default_mode = mode_str.parse()?;
    }
    if let Some(inc) = args.include {
        config.include_pattern = inc;
    }
    if let Some(exc) = args.exclude {
        config.exclude_pattern = exc;
    }
    if let Some(lang) = args.lang {
        config.language = lang;
    }

    let files = scanner::scan(&args.path, &config)?;

    let mut cache = if config.cache_enabled && !args.no_cache {
        match cache::Cache::load() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: failed to load cache ({}), using empty cache", e);
                cache::Cache::new()
            }
        }
    } else {
        cache::Cache::new()
    };

    let descriptions = match config.default_mode {
        Mode::Llm => analyzer::llm::describe_files(&files, &config, &mut cache)?,
        Mode::Pattern => analyzer::pattern::describe_files(&files, &mut cache),
    };

    if config.cache_enabled && !args.no_cache {
        if let Err(e) = cache.save() {
            eprintln!("Warning: failed to save cache: {}", e);
        }
    }

    render::print_tree(&descriptions, args.depth, args.format);

    Ok(())
}
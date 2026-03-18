mod analyzer;
mod cache;
mod config;
mod dump;
mod render;
mod scanner;
mod types;

use crate::config::{Config, Mode};
use crate::types::{Language, OutputFormat};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ai-dir",
    about = "Generate one-line descriptions for files in a directory",
    version
)]
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

    // Dump options
    #[arg(long)]
    dump: bool,
    #[arg(long, value_name = "BYTES")]
    max_size: Option<u64>,
    #[arg(long, value_name = "LINES")]
    max_lines: Option<usize>,
    #[arg(long)]
    include_binary: bool,
    #[arg(long, short = 'q')]
    quiet: bool,
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

    if args.dump {
        dump::dump_files(
            &files,
            args.max_size,
            args.max_lines,
            args.include_binary,
            args.quiet,
        )?;
        return Ok(());
    }

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

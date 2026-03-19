// src/main.rs
mod cli;
mod commands;
mod core;

use anyhow::Result;
use clap::Parser;
use core::config::{Config, Mode};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = cli::Args::parse();

    if args.diff {
        commands::diff::run(args.staged)?;
        return Ok(());
    }

    let global_config = Config::load()?;
    let local_config_path = args.path.join(".ai-dir.toml");
    let local_config = if local_config_path.exists() {
        Some(toml::from_str(&std::fs::read_to_string(
            local_config_path,
        )?)?)
    } else {
        None
    };

    let mut config = local_config
        .map(|c| global_config.merge(c))
        .unwrap_or(global_config);

    if let Some(mode_str) = args.mode {
        config.default_mode = mode_str.parse::<Mode>()?;
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

    if args.refresh_cache {
        let cache_path = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ai-dir")
            .join("cache.json");
        if cache_path.exists() {
            std::fs::remove_file(&cache_path)?;
            eprintln!("Cache cleared.");
        }
    }

    let files = core::scanner::scan(&args.path, &config)?;

    if let Some(format) = args.dump {
        let dump_format = match format {
            cli::DumpFormat::Plain => commands::dump::DumpFormat::Plain,
            cli::DumpFormat::Markdown => commands::dump::DumpFormat::Markdown,
            cli::DumpFormat::Xml => commands::dump::DumpFormat::Xml,
        };
        commands::dump::run(
            &files,
            args.max_size,
            args.max_lines,
            args.include_binary,
            args.quiet,
            args.detailed,
            args.absolute_paths,
            dump_format,
            &args.path,
        )?;
        return Ok(());
    }

    let mut cache = if config.cache_enabled && !args.no_cache {
        core::cache::Cache::load().unwrap_or_else(|e| {
            eprintln!("Warning: failed to load cache ({}), using empty cache", e);
            core::cache::Cache::new()
        })
    } else {
        core::cache::Cache::new()
    };

    let descriptions = match config.default_mode {
        Mode::Llm => commands::analyze::describe_files_llm(&files, &config, &mut cache)?,
        Mode::Pattern => {
            commands::analyze::describe_files_pattern(&files, &mut cache, args.no_truncate)
        }
    };

    if config.cache_enabled && !args.no_cache {
        if let Err(e) = cache.save() {
            eprintln!("Warning: failed to save cache: {}", e);
        }
    }

    core::render::print_tree(
        &descriptions,
        args.depth,
        args.format,
        args.long,
        args.no_long_indicator,
    );

    Ok(())
}

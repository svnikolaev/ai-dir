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
    version,
    after_help = "EXAMPLES:\n\
                  \x20  # Basic pattern mode (default)\n\
                  \x20  aid\n\
                  \n\
                  \x20  # LLM mode with Russian descriptions\n\
                  \x20  aid -m llm --lang ru /path/to/project\n\
                  \n\
                  \x20  # Dump mode (plain format, default)\n\
                  \x20  aid --dump . > all.txt\n\
                  \n\
                  \x20  # Dump in Markdown format with metadata\n\
                  \x20  aid --dump markdown --detailed src/ > docs.md\n\
                  \n\
                  \x20  # Dump in XML with absolute paths\n\
                  \x20  aid --dump xml --absolute-paths . > project.xml\n\
                  \n\
                  \x20  # Limit file size and lines per file\n\
                  \x20  aid --dump --max-size 1M --max-lines 50 .\n\
                  \n\
                  \x20  # Include binary files (use with caution)\n\
                  \x20  aid --dump --include-binary .\n\
                  \n\
                  \x20  # Quiet mode (suppress warnings)\n\
                  \x20  aid --dump --quiet ."
)]
struct Args {
    #[arg(default_value = ".", help = "Directory to analyze")]
    path: PathBuf,

    #[arg(
        short,
        long,
        help = "Analysis mode: 'pattern' (fast, regex-based) or 'llm' (AI-generated descriptions)"
    )]
    mode: Option<String>,

    #[arg(
        short,
        long,
        help = "Maximum depth of directory tree to display (only for tree output)"
    )]
    depth: Option<usize>,

    #[arg(long, value_enum, default_value_t = OutputFormat::Color, help = "Output format for tree mode: plain, color, json")]
    format: OutputFormat,

    #[arg(
        long,
        help = "Regular expression to include files (e.g., '\\.(rs|md)$')"
    )]
    include: Option<String>,

    #[arg(
        long,
        help = "Regular expression to exclude files (e.g., 'target|node_modules')"
    )]
    exclude: Option<String>,

    #[arg(
        long,
        help = "Disable cache (bypass reading/writing cached descriptions)"
    )]
    no_cache: bool,

    #[arg(
        long,
        value_enum,
        help = "Language for LLM-generated descriptions: 'en' or 'ru'"
    )]
    lang: Option<Language>,

    // Dump options
    #[arg(
        long,
        default_missing_value = "plain",
        num_args(0..=1),
        value_enum,
        help = "Dump file contents instead of generating descriptions. Optionally specify format: plain, markdown, xml (default: plain)"
    )]
    dump: Option<DumpFormat>,

    #[arg(
        long,
        value_name = "BYTES",
        help = "Skip files larger than this size (e.g., '1048576', '1M')"
    )]
    max_size: Option<u64>,

    #[arg(
        long,
        value_name = "LINES",
        help = "Output only first N lines of each file (adds [...truncated...] marker)"
    )]
    max_lines: Option<usize>,

    #[arg(
        long,
        help = "Attempt to read binary files as text (replaces invalid UTF-8 sequences)"
    )]
    include_binary: bool,

    #[arg(
        short = 'q',
        long,
        help = "Suppress warning messages (errors still go to stderr as markers)"
    )]
    quiet: bool,

    #[arg(
        long,
        help = "Add metadata (size, modification time) to headers (plain, markdown)"
    )]
    detailed: bool,

    #[arg(
        long,
        help = "Use absolute paths instead of relative in headers/attributes"
    )]
    absolute_paths: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum DumpFormat {
    Plain,
    Markdown,
    Xml,
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

    if let Some(format) = args.dump {
        let dump_format = match format {
            DumpFormat::Plain => dump::DumpFormat::Plain,
            DumpFormat::Markdown => dump::DumpFormat::Markdown,
            DumpFormat::Xml => dump::DumpFormat::Xml,
        };
        dump::dump_files(
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

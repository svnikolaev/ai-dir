mod commands;
mod core;

use anyhow::Result;
use clap::Parser;
use core::config::{Config, Mode};
use core::types::{Language, OutputFormat};
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
                  \x20  aid --dump --quiet .\n\
                  \n\
                  \x20  # Show git diff --staged (for commit messages)\n\
                  \x20  aid --diff\n\
                  \x20  aid --diff --staged"
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
        short = 'D',
        long,
        help = "Add metadata (size, modification time) to headers (plain, markdown)"
    )]
    detailed: bool,

    #[arg(
        short = 'A',
        long,
        help = "Use absolute paths instead of relative in headers/attributes"
    )]
    absolute_paths: bool,

    // Diff options
    #[arg(short = 'g', long, help = "Show git diff (like 'git diff')")]
    diff: bool,

    #[arg(
        short = 's',
        long,
        help = "Show staged changes (equivalent to 'git diff --staged')"
    )]
    staged: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum DumpFormat {
    Plain,
    #[value(alias = "md")]
    Markdown,
    Xml,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Если запрошен diff, запускаем его и выходим
    if args.diff {
        commands::diff::run(args.staged)?;
        return Ok(());
    }

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

    let files = core::scanner::scan(&args.path, &config)?;

    if let Some(format) = args.dump {
        let dump_format = match format {
            DumpFormat::Plain => commands::dump::DumpFormat::Plain,
            DumpFormat::Markdown => commands::dump::DumpFormat::Markdown,
            DumpFormat::Xml => commands::dump::DumpFormat::Xml,
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

    // Режим analyze
    let mut cache = if config.cache_enabled && !args.no_cache {
        match core::cache::Cache::load() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: failed to load cache ({}), using empty cache", e);
                core::cache::Cache::new()
            }
        }
    } else {
        core::cache::Cache::new()
    };

    let descriptions = match config.default_mode {
        Mode::Llm => commands::analyze::describe_files_llm(&files, &config, &mut cache)?,
        Mode::Pattern => commands::analyze::describe_files_pattern(&files, &mut cache),
    };

    if config.cache_enabled && !args.no_cache {
        if let Err(e) = cache.save() {
            eprintln!("Warning: failed to save cache: {}", e);
        }
    }

    core::render::print_tree(&descriptions, args.depth, args.format);

    Ok(())
}

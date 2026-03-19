use crate::core::types::{Language, OutputFormat};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ai-dir",
    about = "Generate one-line descriptions for files in a directory",
    version,
    after_help = "EXAMPLES:\n\
                  \x20  # Basic pattern mode\n\x20    aid\n\
                  \x20  # Show only long functions (threshold 20)\n\x20    aid --long\n\
                  \x20  # Show functions longer than 40 lines\n\x20    aid --long 40\n\
                  \x20  # LLM mode with Russian\n\x20    aid -m llm --lang ru /path\n\
                  \x20  # Dump plain\n\x20    aid --dump . > all.txt\n\
                  \x20  # Dump markdown with metadata\n\x20    aid --dump markdown --detailed src/ > docs.md\n\
                  \x20  # Dump XML\n\x20    aid --dump xml . > project.xml\n\
                  \x20  # Limit size/lines\n\x20    aid --dump --max-size 1M --max-lines 50 .\n\
                  \x20  # Git diff (staged)\n\x20    aid -gs\n\
                  \x20  # Clear cache\n\x20    aid --refresh-cache\n\
                  \x20  # Full symbols in JSON\n\x20    aid --format json -T"
)]
pub struct Args {
    #[arg(default_value = ".", help = "Directory to analyze")]
    pub path: PathBuf,

    #[arg(short, long, help = "Analysis mode: 'pattern' or 'llm'")]
    pub mode: Option<String>,

    #[arg(short, long, help = "Maximum depth of directory tree")]
    pub depth: Option<usize>,

    #[arg(long, value_enum, default_value_t = OutputFormat::Color, help = "Output format for tree mode")]
    pub format: OutputFormat,

    #[arg(long, help = "Regular expression to include files")]
    pub include: Option<String>,

    #[arg(long, help = "Regular expression to exclude files")]
    pub exclude: Option<String>,

    #[arg(long, help = "Disable cache")]
    pub no_cache: bool,

    #[arg(long, value_enum, help = "Language for LLM descriptions")]
    pub lang: Option<Language>,

    // Dump options
    #[arg(
        long,
        default_missing_value = "plain",
        num_args(0..=1),
        value_enum,
        help = "Dump file contents. Optionally specify format: plain, markdown, xml (default: plain)"
    )]
    pub dump: Option<DumpFormat>,

    #[arg(long, value_name = "BYTES", help = "Skip files larger than this size")]
    pub max_size: Option<u64>,

    #[arg(
        long,
        value_name = "LINES",
        help = "Output only first N lines of each file"
    )]
    pub max_lines: Option<usize>,

    #[arg(long, help = "Attempt to read binary files as text")]
    pub include_binary: bool,

    #[arg(short = 'q', long, help = "Suppress warning messages")]
    pub quiet: bool,

    #[arg(
        short = 'D',
        long,
        help = "Add metadata (size, modification time) to headers"
    )]
    pub detailed: bool,

    #[arg(short = 'A', long, help = "Use absolute paths instead of relative")]
    pub absolute_paths: bool,

    // Diff options
    #[arg(short = 'g', long, help = "Show git diff (unstaged changes)")]
    pub diff: bool,

    #[arg(short = 's', long, help = "Show staged changes (git diff --staged)")]
    pub staged: bool,

    // Long functions threshold
    #[arg(short = 'l', long, default_missing_value = "20", num_args(0..=1), value_name = "LINES", help = "Show only files with functions longer than LINES (default: 20)")]
    pub long: Option<usize>,

    // Cache control
    #[arg(
        short = 'R',
        long,
        help = "Clear cache before running (forces re-analysis)"
    )]
    pub refresh_cache: bool,

    // Display options
    #[arg(
        short = 'T',
        long,
        help = "Do not truncate symbol list (show all symbols)"
    )]
    pub no_truncate: bool,

    #[arg(long, help = "Hide long functions indicator (long: N) from output")]
    pub no_long_indicator: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum DumpFormat {
    Plain,
    #[value(alias = "md")]
    Markdown,
    Xml,
}

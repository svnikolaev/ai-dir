use crate::core::types::{Language, OutputFormat};
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
                  \x20  # Show only files with long functions\n\
                  \x20  aid --long\n\
                  \n\
                  \x20  # Show all symbols without truncation\n\
                  \x20  aid --no-truncate\n\
                  \n\
                  \x20  # Hide long functions indicator in file lines\n\
                  \x20  aid --no-long-indicator\n\
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
                  \x20  # Show git diff (unstaged changes)\n\
                  \x20  aid --diff\n\
                  \n\
                  \x20  # Show staged changes (for commit messages)\n\
                  \x20  aid --diff --staged\n\
                  \n\
                  \x20  # Clear cache before running\n\
                  \x20  aid --refresh-cache"
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

    // Long functions
    #[arg(
        short = 'l',
        long,
        help = "Show only files with functions longer than threshold"
    )]
    pub long: bool,

    // Cache control
    #[arg(
        short = 'R',
        long,
        help = "Clear cache before running (forces re-analysis)"
    )]
    pub refresh_cache: bool,

    // Display options
    #[arg(long, help = "Do not truncate symbol list (show all symbols)")]
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

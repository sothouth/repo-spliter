use std::sync::LazyLock;

use clap::{Parser, ValueEnum};

#[inline(always)]
pub fn cli() -> &'static Cli {
    static CLI: LazyLock<Cli> = LazyLock::new(Cli::parse);
    &CLI
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(help = "Path to source Git repository")]
    pub repo: String,
    #[clap(help = "Relative path to target subdirectory")]
    pub path: String,
    #[clap(value_enum, default_value_t = Remove::Nothing,help="\
Post-split cleanup action [default: nothing]
    Possible values:
        -n nothing: Preserve original directory
        -c commit: Remove directory in new commit
        -p prune: Purge directory from history")]
    pub remove: Remove,
    #[clap(long, short, help = "Output path for new repository")]
    pub local: Option<String>,
    #[clap(long, short, help = "Remote repository URL to set")]
    pub remote: Option<String>,
    #[clap(
        long,
        short,
        help = "Convert original directory to submodule (requires removal)"
    )]
    pub keep: bool,
}

#[derive(Debug, ValueEnum, Default, Clone, Copy)]
pub enum Remove {
    #[default]
    #[clap(alias = "n")]
    Nothing,
    #[clap(alias = "c")]
    Commit,
    #[clap(alias = "p")]
    Prune,
}

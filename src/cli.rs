use std::sync::LazyLock;

use clap::{Parser, ValueEnum};

#[inline(always)]
pub fn cli() -> &'static Cli {
    static CLI: LazyLock<Cli> = LazyLock::new(Cli::parse);
    &CLI
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(help = "The path to the git repository")]
    pub repo: String,
    #[clap(help = "rel-path to the sub-dir")]
    pub path: String,
    #[clap(value_enum, default_value_t = Remove::Nothing,help="remove the sub-dir after split")]
    pub remove: Remove,
    #[clap(long, short, help = "local new repo path")]
    pub local: Option<String>,
    #[clap(long, short, help = "remote new repo path")]
    pub remote: Option<String>,
    #[clap(
        long,
        short,
        help = "make old dir a submodule (if you want to keep it, you need remove it too)"
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

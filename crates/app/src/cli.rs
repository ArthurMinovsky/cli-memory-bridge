use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cli-memory")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Refresh,
    Install { provider: String },
    Unlink {
        provider: Option<String>,
        #[arg(long)]
        all: bool,
    },
    Uninstall,
    Serve,
    Resume { hash_id: String },
    Forget { provider: String, hash_id: String },
    #[command(name = "conv-search")]
    ConvSearch { query: String },
    Doctor,
    Stats,
}

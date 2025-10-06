use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wdlparse::OutputFormat;

mod commands;
mod info;

#[derive(Parser)]
#[command(name = "wdlparse")]
#[command(about = "A command-line tool for parsing WDL (Workflow Description Language) files")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a WDL file and display the syntax tree
    Parse {
        /// Path to the WDL file to parse
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "tree")]
        format: OutputFormat,

        /// Show detailed diagnostic information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show information about a WDL file (version, tasks, workflows, etc.)
    Info {
        /// Path to the WDL file to analyze
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: OutputFormat,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse {
            file,
            format,
            verbose,
        } => commands::parse_command(file, format, verbose),
        Commands::Info { file, format } => commands::info_command(file, format),
    }
}

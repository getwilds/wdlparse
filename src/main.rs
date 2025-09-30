use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod info;
mod mermaid;

#[derive(Parser)]
#[command(name = "wdlparse")]
#[command(about = "A command-line tool for parsing WDL (Workflow Description Language) files")]
#[command(
    long_about = "Parse, analyze, and visualize WDL workflow files.\n\nSupports parsing WDL syntax trees, extracting semantic information, and generating Mermaid diagrams for workflow visualization."
)]
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
    /// Generate a Mermaid diagram from a WDL workflow
    #[command(
        long_about = "Generate a Mermaid.js flowchart diagram from a WDL workflow.\n\nThe diagram shows tasks, workflows, calls, conditionals, scatter operations, and their dependencies. Output can be saved to a file or printed to stdout for use with Mermaid.js renderers."
    )]
    Mermaid {
        /// Path to the WDL file to visualize
        #[arg(value_name = "FILE", help = "WDL file to generate diagram from")]
        file: PathBuf,

        /// Output the diagram to a file instead of stdout
        #[arg(short, long, help = "Write diagram to file (use .mmd extension)")]
        output: Option<PathBuf>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
    /// Syntax tree format
    Tree,
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
        Commands::Mermaid { file, output } => commands::mermaid_command(file, output),
    }
}

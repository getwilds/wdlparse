use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod commands;

#[derive(Default, Serialize, Deserialize)]
pub struct WdlInfo {
    pub version: Option<String>,
    pub tasks: Vec<TaskInfo>,
    pub workflows: Vec<WorkflowInfo>,
    pub structs: Vec<StructInfo>,
    pub imports: Vec<ImportInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct TaskInfo {
    pub name: String,
    pub inputs: Vec<InputInfo>,
    pub outputs: Vec<OutputInfo>,
    pub command: Option<String>,
    pub runtime: Vec<RuntimeItem>,
    pub meta: Vec<MetaItem>,
    pub parameter_meta: Vec<MetaItem>,
}

#[derive(Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub name: String,
    pub inputs: Vec<InputInfo>,
    pub outputs: Vec<OutputInfo>,
    pub calls: Vec<CallInfo>,
    pub meta: Vec<MetaItem>,
    pub parameter_meta: Vec<MetaItem>,
}

#[derive(Serialize, Deserialize)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<InputInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct ImportInfo {
    pub uri: String,
    pub alias: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct InputInfo {
    pub name: String,
    pub wdl_type: String,
    pub optional: bool,
    pub default_value: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct OutputInfo {
    pub name: String,
    pub wdl_type: String,
    pub expression: String,
}

#[derive(Serialize, Deserialize)]
pub struct CallInfo {
    pub name: String,
    pub target: String,
    pub alias: Option<String>,
    pub inputs: Vec<CallInputItem>,
}

#[derive(Serialize, Deserialize)]
pub struct CallInputItem {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct RuntimeItem {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct MetaItem {
    pub key: String,
    pub value: String,
}

impl WdlInfo {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Parser)]
#[command(name = "wdlparse")]
#[command(about = "A command-line tool for parsing WDL (Workflow Description Language) files")]
#[command(version)]
struct Cli {
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

#[derive(clap::ValueEnum, Clone, Debug)]
enum OutputFormat {
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
    }
}

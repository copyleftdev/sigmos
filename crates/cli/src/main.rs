//! # SIGMOS CLI
//!
//! Command-line interface for the SIGMOS DSL.
//!
//! Provides commands for validating, transpiling, running, and managing SIGMOS specifications.
//!
//! # Examples
//!
//! ```bash
//! sigmos validate spec.sigmos
//! sigmos run spec.sigmos
//! sigmos transpile spec.sigmos --to json
//! ```

use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result};
use sigmos_core::parser::SigmosParser;
use sigmos_runtime::Runtime;
use sigmos_transpiler::Transpiler;
use std::path::PathBuf;
use tokio;

/// SIGMOS: Sigma Modular Operating Spec CLI
#[derive(Parser)]
#[command(name = "sigmos")]
#[command(about = "A next-generation DSL for AI-native, composable, reactive systems")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a SIGMOS specification
    Validate {
        /// Path to the SIGMOS specification file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Run a SIGMOS specification
    Run {
        /// Path to the SIGMOS specification file
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Runtime configuration options
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Transpile a SIGMOS specification to another format
    Transpile {
        /// Path to the SIGMOS specification file
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Output format
        #[arg(long, value_enum, default_value = "json")]
        to: OutputFormat,
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Install a SIGMOS plugin
    Install {
        /// Plugin name or path
        #[arg(value_name = "PLUGIN")]
        plugin: String,
    },
    /// Create a new SIGMOS plugin scaffold
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },
    /// Describe or explain a SIGMOS specification
    Describe {
        /// Path to the SIGMOS specification file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// Create a new plugin
    New {
        /// Plugin name
        #[arg(value_name = "NAME")]
        name: String,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    Json,
    Yaml,
    Toml,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { file } => {
            validate_spec(&file).await?
        }
        Commands::Run { file, config } => {
            run_spec(&file, config.as_ref()).await?
        }
        Commands::Transpile { file, to, output } => {
            transpile_spec(&file, to, output.as_ref()).await?
        }
        Commands::Install { plugin } => {
            install_plugin(&plugin).await?
        }
        Commands::Plugin { command } => {
            match command {
                PluginCommands::New { name } => {
                    create_plugin_scaffold(&name).await?
                }
            }
        }
        Commands::Describe { file } => {
            describe_spec(&file).await?
        }
    }

    Ok(())
}

async fn validate_spec(file: &PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(file)
        .into_diagnostic()
        .map_err(|e| miette::miette!("Failed to read file {}: {}", file.display(), e))?;

    let spec = SigmosParser::parse_spec(&content).into_diagnostic()?;

    println!("✓ Specification '{}' v{} is valid", spec.name, spec.version);
    Ok(())
}

async fn run_spec(file: &PathBuf, _config: Option<&PathBuf>) -> Result<()> {
    let content = std::fs::read_to_string(file)
        .into_diagnostic()
        .map_err(|e| miette::miette!("Failed to read file {}: {}", file.display(), e))?;

    let spec = SigmosParser::parse_spec(&content).into_diagnostic()?;

    let mut runtime = Runtime::new();
    runtime.execute(&spec).await
        .map_err(|e| miette::miette!("Runtime error: {}", e))?;

    Ok(())
}

async fn transpile_spec(file: &PathBuf, format: OutputFormat, output: Option<&PathBuf>) -> Result<()> {
    let content = std::fs::read_to_string(file)
        .into_diagnostic()
        .map_err(|e| miette::miette!("Failed to read file {}: {}", file.display(), e))?;

    let spec = SigmosParser::parse_spec(&content).into_diagnostic()?;

    let transpiler = Transpiler::new();
    let result = match format {
        OutputFormat::Json => transpiler.to_json(&spec).map_err(|e| miette::miette!("Transpiler error: {}", e))?,
        OutputFormat::Yaml => transpiler.to_yaml(&spec).map_err(|e| miette::miette!("Transpiler error: {}", e))?,
        OutputFormat::Toml => transpiler.to_toml(&spec).map_err(|e| miette::miette!("Transpiler error: {}", e))?,
    };

    match output {
        Some(output_file) => {
            std::fs::write(output_file, result)
                .into_diagnostic()
                .map_err(|e| miette::miette!("Failed to write output file {}: {}", output_file.display(), e))?;
            println!("Transpiled to {}", output_file.display());
        }
        None => {
            println!("{}", result);
        }
    }

    Ok(())
}

async fn install_plugin(_plugin: &str) -> Result<()> {
    println!("Plugin installation not yet implemented");
    Ok(())
}

async fn create_plugin_scaffold(_name: &str) -> Result<()> {
    println!("Plugin scaffold creation not yet implemented");
    Ok(())
}

async fn describe_spec(file: &PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(file)
        .into_diagnostic()
        .map_err(|e| miette::miette!("Failed to read file {}: {}", file.display(), e))?;

    let spec = SigmosParser::parse_spec(&content).into_diagnostic()?;

    println!("Specification: {} v{}", spec.name, spec.version);
    if let Some(desc) = &spec.description {
        println!("Description: {}", desc);
    }
    println!("Inputs: {} fields", spec.inputs.len());
    println!("Computed: {} fields", spec.computed.len());
    println!("Events: {} handlers", spec.events.len());
    println!("Constraints: {} rules", spec.constraints.len());
    println!("Extensions: {} plugins", spec.extensions.len());
    
    Ok(())
}

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

async fn install_plugin(plugin: &str) -> Result<()> {
    println!("Installing plugin: {}", plugin);
    
    // Check if it's a local path or a plugin name
    let plugin_path = std::path::Path::new(plugin);
    
    if plugin_path.exists() {
        // Local plugin installation
        install_local_plugin(plugin_path).await
    } else {
        // Registry plugin installation
        install_registry_plugin(plugin).await
    }
}

async fn install_local_plugin(plugin_path: &std::path::Path) -> Result<()> {
    println!("Installing local plugin from: {}", plugin_path.display());
    
    // Validate plugin structure
    let cargo_toml = plugin_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(miette::miette!("Invalid plugin: Cargo.toml not found in {}", plugin_path.display()));
    }
    
    // Read and validate Cargo.toml
    let cargo_content = std::fs::read_to_string(&cargo_toml)
        .into_diagnostic()
        .map_err(|e| miette::miette!("Failed to read Cargo.toml: {}", e))?;
    
    // Check if it's a valid SIGMOS plugin by looking for sigmos-core dependency
    if !cargo_content.contains("sigmos-core") {
        return Err(miette::miette!("Invalid SIGMOS plugin: missing sigmos-core dependency"));
    }
    
    // Build the plugin
    println!("Building plugin...");
    let output = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(plugin_path)
        .output()
        .into_diagnostic()
        .map_err(|e| miette::miette!("Failed to build plugin: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(miette::miette!("Plugin build failed:\n{}", stderr));
    }
    
    println!("✓ Plugin installed successfully from {}", plugin_path.display());
    println!("Note: Plugin is built but not yet integrated into the runtime registry.");
    println!("To use the plugin, ensure it's properly registered in your SIGMOS specifications.");
    
    Ok(())
}

async fn install_registry_plugin(plugin_name: &str) -> Result<()> {
    println!("Installing plugin from registry: {}", plugin_name);
    
    // Check if it's a known built-in plugin
    match plugin_name {
        "mcp" | "rest" => {
            println!("✓ Plugin '{}' is already available as a built-in plugin.", plugin_name);
            println!("You can use it directly in your SIGMOS specifications.");
            return Ok(());
        }
        _ => {}
    }
    
    // For now, provide guidance on how to add external plugins
    println!("External plugin registry not yet implemented.");
    println!("To install external plugins:");
    println!("1. Clone the plugin repository locally");
    println!("2. Run: sigmos install /path/to/plugin");
    println!("3. Or add the plugin as a dependency in your project's Cargo.toml");
    
    Ok(())
}

async fn create_plugin_scaffold(name: &str) -> Result<()> {
    println!("Creating plugin scaffold: {}", name);
    
    // Validate plugin name
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(miette::miette!("Invalid plugin name. Use only alphanumeric characters, hyphens, and underscores."));
    }
    
    let plugin_dir = std::path::Path::new(name);
    
    // Check if directory already exists
    if plugin_dir.exists() {
        return Err(miette::miette!("Directory '{}' already exists", name));
    }
    
    // Create plugin directory structure
    std::fs::create_dir_all(plugin_dir.join("src"))
        .into_diagnostic()
        .map_err(|e| miette::miette!("Failed to create plugin directory: {}", e))?;
    
    // Create Cargo.toml
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
license = "MIT OR Apache-2.0"
description = "A SIGMOS plugin for {}"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
sigmos-core = {{ path = "../sigmos/crates/core" }}
sigmos-runtime = {{ path = "../sigmos/crates/runtime" }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
thiserror = "1.0"
tokio = {{ version = "1.0", features = ["full"] }}

[dev-dependencies]
tokio-test = "0.4"
"#, name, name);
    
    std::fs::write(plugin_dir.join("Cargo.toml"), cargo_toml)
        .into_diagnostic()
        .map_err(|e| miette::miette!("Failed to create Cargo.toml: {}", e))?;
    
    // Create lib.rs with plugin template
    let lib_rs = format!(r#"//! # {} Plugin
//!
//! A SIGMOS plugin for {}.

use serde::{{Deserialize, Serialize}};
use serde_json::Value as JsonValue;
use sigmos_core::{{Plugin, PluginConfig, PluginError, ConfigurablePlugin}};
use sigmos_runtime::{{RuntimeResult, RuntimeError}};
use std::collections::HashMap;
use thiserror::Error;

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {}Config {{
    pub name: String,
    // Add your configuration fields here
}}

impl PluginConfig for {}Config {{
    fn validate(&self) -> Result<(), PluginError> {{
        if self.name.is_empty() {{
            return Err(PluginError::InvalidConfig("Plugin name cannot be empty".to_string()));
        }}
        Ok(())
    }}
    
    fn plugin_name(&self) -> &str {{
        &self.name
    }}
}}

impl Default for {}Config {{
    fn default() -> Self {{
        Self {{
            name: "{}".to_string(),
        }}
    }}
}}

/// {} Plugin
#[derive(Debug)]
pub struct {}Plugin {{
    config: {}Config,
    initialized: bool,
}}

impl ConfigurablePlugin for {}Plugin {{
    type Config = {}Config;
    
    fn new(config: Self::Config) -> Result<Self, PluginError> {{
        config.validate()?;
        Ok({}Plugin {{
            config,
            initialized: false,
        }})
    }}
    
    fn config(&self) -> &Self::Config {{
        &self.config
    }}
    
    fn update_config(&mut self, config: Self::Config) -> Result<(), PluginError> {{
        config.validate()?;
        self.config = config;
        Ok(())
    }}
}}

impl Plugin for {}Plugin {{
    fn name(&self) -> &str {{
        &self.config.name
    }}
    
    fn initialize(&mut self) -> RuntimeResult<()> {{
        // Initialize your plugin here
        self.initialized = true;
        println!("Initialized {} plugin");
        Ok(())
    }}
    
    fn execute(
        &self,
        method: &str,
        args: &HashMap<String, JsonValue>,
    ) -> RuntimeResult<JsonValue> {{
        if !self.initialized {{
            return Err(RuntimeError::Plugin("{} plugin not initialized".to_string()));
        }}
        
        match method {{
            "hello" => self.hello(args),
            // Add your plugin methods here
            _ => Err(RuntimeError::Plugin(format!("Unknown method: {{}}", method))),
        }}
    }}
}}

impl {}Plugin {{
    /// Example method - replace with your plugin functionality
    fn hello(&self, args: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue> {{
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");
        
        let response = format!("Hello, {{}} from {} plugin!", name);
        Ok(JsonValue::String(response))
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_plugin_creation() {{
        let config = {}Config::default();
        let plugin = {}Plugin::new(config);
        assert!(plugin.is_ok());
    }}
    
    #[tokio::test]
    async fn test_plugin_execution() {{
        let config = {}Config::default();
        let mut plugin = {}Plugin::new(config).unwrap();
        assert!(plugin.initialize().is_ok());
        
        let mut args = HashMap::new();
        args.insert("name".to_string(), JsonValue::String("SIGMOS".to_string()));
        
        let result = plugin.execute("hello", &args);
        assert!(result.is_ok());
    }}
}}
"#, 
        name.replace('-', "_").replace('_', " ").split(' ').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(" "),
        name,
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name,
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(" "),
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name,
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name,
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
        name.replace('-', "_").split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(""),
    );
    
    std::fs::write(plugin_dir.join("src").join("lib.rs"), lib_rs)
        .into_diagnostic()
        .map_err(|e| miette::miette!("Failed to create lib.rs: {}", e))?;
    
    // Create README.md
    let readme = format!(r#"# {} Plugin

A SIGMOS plugin for {}.

## Installation

```bash
# Build the plugin
cargo build --release

# Install via SIGMOS CLI
sigmos install .
```

## Usage

Add the plugin to your SIGMOS specification:

```sigmos
spec "MySpec" v1.0 {{
  plugins: [
    {{
      name: "{}"
      config: {{
        // Plugin configuration
      }}
    }}
  ]
  
  // Use plugin methods
  computed: {{
    greeting: {}.hello({{ name: "World" }})
  }}
}}
```

## Development

```bash
# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy
```
"#, 
        name.replace('-', " ").split(' ').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<_>>().join(" "),
        name,
        name,
        name
    );
    
    std::fs::write(plugin_dir.join("README.md"), readme)
        .into_diagnostic()
        .map_err(|e| miette::miette!("Failed to create README.md: {}", e))?;
    
    println!("✓ Plugin scaffold created successfully!");
    println!("Directory: {}", plugin_dir.display());
    println!("");
    println!("Next steps:");
    println!("1. cd {}", name);
    println!("2. Edit src/lib.rs to implement your plugin functionality");
    println!("3. cargo test  # Run tests");
    println!("4. cargo build --release  # Build the plugin");
    println!("5. sigmos install .  # Install the plugin");
    
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

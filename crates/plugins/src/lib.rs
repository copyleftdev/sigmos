//! SIGMOS Plugin System
//!
//! This crate provides the official plugin system for SIGMOS, including:
//! - Core plugin traits and utilities
//! - MCP (Model Context Protocol) plugin for AI integration
//! - REST plugin for HTTP/API interactions
//! - Plugin registry and management
//!
//! # Usage Example
//!
//! ```rust
//! use sigmos_plugins::{
//!     registry::PluginRegistry,
//!     mcp::{McpPlugin, McpConfig},
//!     rest::{RestPlugin, RestConfig},
//!     ConfigurablePlugin,
//! };
//! use std::collections::HashMap;
//! use serde_json::Value as JsonValue;
//!
//! // Create plugin registry
//! let mut registry = PluginRegistry::new();
//!
//! // Configure and register MCP plugin
//! let mcp_config = McpConfig {
//!     name: "ai_assistant".to_string(),
//!     endpoint: "https://api.openai.com/v1".to_string(),
//!     model: "gpt-4".to_string(),
//!     api_key: Some("your-api-key".to_string()),
//!     ..Default::default()
//! };
//! let mcp_plugin = McpPlugin::new(mcp_config).unwrap();
//! registry.register_plugin(
//!     Box::new(mcp_plugin),
//!     sigmos_plugins::mcp::McpPlugin::metadata(),
//!     sigmos_plugins::mcp::McpPlugin::capabilities(),
//! ).unwrap();
//!
//! // Configure and register REST plugin
//! let rest_config = RestConfig {
//!     name: "api_client".to_string(),
//!     base_url: "https://jsonplaceholder.typicode.com".to_string(),
//!     ..Default::default()
//! };
//! let rest_plugin = RestPlugin::new(rest_config).unwrap();
//! registry.register_plugin(
//!     Box::new(rest_plugin),
//!     sigmos_plugins::rest::RestPlugin::metadata(),
//!     sigmos_plugins::rest::RestPlugin::capabilities(),
//! ).unwrap();
//!
//! // Verify plugins are registered
//! assert!(registry.is_plugin_enabled("ai_assistant"));
//! assert!(registry.is_plugin_enabled("api_client"));
//!
//! args.clear();
//! args.insert("path".to_string(), JsonValue::String("/posts/1".to_string()));
//! let api_response = registry.execute_plugin_method("api_client", "get", &args).unwrap();
//! ```

use sigmos_runtime::Plugin;
use thiserror::Error;

pub mod mcp;
pub mod rest;
pub mod registry;

/// Plugin system errors
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Plugin method not found: {0}")]
    MethodNotFound(String),
    #[error("Plugin execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Invalid plugin configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Plugin configuration trait
pub trait PluginConfig: std::fmt::Debug + Clone {
    /// Validate the configuration
    fn validate(&self) -> Result<(), PluginError>;
    
    /// Get plugin name from configuration
    fn plugin_name(&self) -> &str;
}

/// Enhanced plugin trait with configuration support
pub trait ConfigurablePlugin: Plugin {
    type Config: PluginConfig;
    
    /// Create a new plugin instance with configuration
    fn new(config: Self::Config) -> Result<Self, PluginError>
    where
        Self: Sized;
    
    /// Get the plugin configuration
    fn config(&self) -> &Self::Config;
    
    /// Update the plugin configuration
    fn update_config(&mut self, config: Self::Config) -> Result<(), PluginError>;
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub methods: Vec<String>,
}

/// Plugin capabilities
#[derive(Debug, Clone)]
pub struct PluginCapabilities {
    pub supports_async: bool,
    pub supports_streaming: bool,
    pub requires_network: bool,
    pub requires_auth: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_error_display() {
        let error = PluginError::MethodNotFound("test_method".to_string());
        assert!(error.to_string().contains("test_method"));
    }
}

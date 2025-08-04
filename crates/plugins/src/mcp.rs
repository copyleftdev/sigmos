//! Model Context Protocol (MCP) Plugin
//!
//! This plugin provides integration with AI models and services through the Model Context Protocol,
//! enabling SIGMOS to interact with language models, embeddings, and other AI services.

use crate::{ConfigurablePlugin, PluginConfig, PluginError, PluginCapabilities, PluginMetadata};
use sigmos_runtime::{Plugin, RuntimeResult, RuntimeError};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// MCP plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub name: String,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub model: String,
    pub timeout_seconds: u64,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

impl PluginConfig for McpConfig {
    fn validate(&self) -> Result<(), PluginError> {
        if self.name.is_empty() {
            return Err(PluginError::InvalidConfiguration("name cannot be empty".to_string()));
        }
        if self.endpoint.is_empty() {
            return Err(PluginError::InvalidConfiguration("endpoint cannot be empty".to_string()));
        }
        if self.model.is_empty() {
            return Err(PluginError::InvalidConfiguration("model cannot be empty".to_string()));
        }
        if let Some(temp) = self.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(PluginError::InvalidConfiguration("temperature must be between 0.0 and 2.0".to_string()));
            }
        }
        Ok(())
    }
    
    fn plugin_name(&self) -> &str {
        &self.name
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            name: "mcp".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            api_key: None,
            model: "gpt-3.5-turbo".to_string(),
            timeout_seconds: 30,
            max_tokens: Some(1000),
            temperature: Some(0.7),
        }
    }
}

/// MCP Plugin for AI model integration
#[derive(Debug)]
pub struct McpPlugin {
    config: McpConfig,
    initialized: bool,
}

impl ConfigurablePlugin for McpPlugin {
    type Config = McpConfig;
    
    fn new(config: Self::Config) -> Result<Self, PluginError> {
        config.validate()?;
        Ok(Self {
            config,
            initialized: false,
        })
    }
    
    fn config(&self) -> &Self::Config {
        &self.config
    }
    
    fn update_config(&mut self, config: Self::Config) -> Result<(), PluginError> {
        config.validate()?;
        self.config = config;
        Ok(())
    }
}

impl Plugin for McpPlugin {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn initialize(&mut self) -> RuntimeResult<()> {
        // Initialize MCP connection
        // For now, this is a placeholder - would normally establish connection to MCP server
        self.initialized = true;
        Ok(())
    }
    
    fn execute(
        &self,
        method: &str,
        args: &HashMap<String, JsonValue>,
    ) -> RuntimeResult<JsonValue> {
        if !self.initialized {
            return Err(RuntimeError::Plugin("MCP plugin not initialized".to_string()));
        }
        
        match method {
            "complete" => self.complete(args),
            "embed" => self.embed(args),
            "chat" => self.chat(args),
            "analyze" => self.analyze(args),
            _ => Err(RuntimeError::Plugin(format!("Unknown MCP method: {}", method))),
        }
    }
}

impl McpPlugin {
    /// Text completion method
    fn complete(&self, args: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue> {
        let prompt = args.get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RuntimeError::Plugin("Missing 'prompt' argument".to_string()))?;
        
        // Placeholder implementation - would normally call MCP service
        let response = format!("MCP completion for: {}", prompt);
        Ok(JsonValue::Object({
            let mut obj = serde_json::Map::new();
            obj.insert("text".to_string(), JsonValue::String(response));
            obj.insert("model".to_string(), JsonValue::String(self.config.model.clone()));
            obj.insert("tokens_used".to_string(), JsonValue::Number(serde_json::Number::from(42)));
            obj
        }))
    }
    
    /// Text embedding method
    fn embed(&self, args: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue> {
        let text = args.get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RuntimeError::Plugin("Missing 'text' argument".to_string()))?;
        
        // Placeholder implementation - would normally generate embeddings
        let embedding: Vec<f32> = (0..768).map(|i| (i as f32) * 0.001).collect();
        Ok(JsonValue::Object({
            let mut obj = serde_json::Map::new();
            obj.insert("embedding".to_string(), JsonValue::Array(
                embedding.into_iter().map(|f| JsonValue::Number(
                    serde_json::Number::from_f64(f as f64).unwrap()
                )).collect()
            ));
            obj.insert("dimensions".to_string(), JsonValue::Number(serde_json::Number::from(768)));
            obj.insert("input_text".to_string(), JsonValue::String(text.to_string()));
            obj
        }))
    }
    
    /// Chat completion method
    fn chat(&self, args: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue> {
        let _messages = args.get("messages")
            .ok_or_else(|| RuntimeError::Plugin("Missing 'messages' argument".to_string()))?;
        
        // Placeholder implementation - would normally handle chat conversation
        Ok(JsonValue::Object({
            let mut obj = serde_json::Map::new();
            obj.insert("response".to_string(), JsonValue::String("MCP chat response".to_string()));
            obj.insert("role".to_string(), JsonValue::String("assistant".to_string()));
            obj.insert("model".to_string(), JsonValue::String(self.config.model.clone()));
            obj
        }))
    }
    
    /// Text analysis method
    fn analyze(&self, args: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue> {
        let text = args.get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RuntimeError::Plugin("Missing 'text' argument".to_string()))?;
        
        // Placeholder implementation - would normally perform text analysis
        Ok(JsonValue::Object({
            let mut obj = serde_json::Map::new();
            obj.insert("sentiment".to_string(), JsonValue::String("positive".to_string()));
            obj.insert("confidence".to_string(), JsonValue::Number(serde_json::Number::from_f64(0.85).unwrap()));
            obj.insert("word_count".to_string(), JsonValue::Number(serde_json::Number::from(text.split_whitespace().count())));
            obj.insert("language".to_string(), JsonValue::String("en".to_string()));
            obj
        }))
    }
    
    /// Get plugin metadata
    pub fn metadata() -> PluginMetadata {
        PluginMetadata {
            name: "mcp".to_string(),
            version: "1.0.0".to_string(),
            description: "Model Context Protocol integration for AI services".to_string(),
            author: "SIGMOS Team".to_string(),
            methods: vec![
                "complete".to_string(),
                "embed".to_string(),
                "chat".to_string(),
                "analyze".to_string(),
            ],
        }
    }
    
    /// Get plugin capabilities
    pub fn capabilities() -> PluginCapabilities {
        PluginCapabilities {
            supports_async: true,
            supports_streaming: true,
            requires_network: true,
            requires_auth: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_config_validation() {
        let mut config = McpConfig::default();
        assert!(config.validate().is_ok());
        
        config.name = "".to_string();
        assert!(config.validate().is_err());
        
        config.name = "test".to_string();
        config.temperature = Some(3.0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_mcp_plugin_creation() {
        let config = McpConfig::default();
        let plugin = McpPlugin::new(config);
        assert!(plugin.is_ok());
    }

    #[test]
    fn test_mcp_plugin_methods() {
        let config = McpConfig::default();
        let mut plugin = McpPlugin::new(config).unwrap();
        assert!(plugin.initialize().is_ok());
        
        let mut args = HashMap::new();
        args.insert("prompt".to_string(), JsonValue::String("Hello world".to_string()));
        
        let result = plugin.execute("complete", &args);
        assert!(result.is_ok());
    }
}

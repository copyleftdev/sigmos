//! Plugin Registry
//!
//! This module provides plugin registration, discovery, and management capabilities
//! for the SIGMOS plugin system.

use crate::{PluginError, PluginMetadata, PluginCapabilities};
use sigmos_runtime::{Plugin, RuntimeResult};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Plugin registry entry
#[derive(Debug)]
pub struct PluginEntry {
    pub metadata: PluginMetadata,
    pub capabilities: PluginCapabilities,
    pub plugin: Arc<RwLock<Box<dyn Plugin>>>,
    pub enabled: bool,
}

/// Plugin registry for managing loaded plugins
#[derive(Debug, Default)]
pub struct PluginRegistry {
    plugins: HashMap<String, PluginEntry>,
    aliases: HashMap<String, String>, // alias -> plugin_name mapping
}

/// Plugin registration info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistrationInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub methods: Vec<String>,
    pub aliases: Vec<String>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a plugin in the registry
    pub fn register_plugin(
        &mut self,
        plugin: Box<dyn Plugin>,
        metadata: PluginMetadata,
        capabilities: PluginCapabilities,
    ) -> Result<(), PluginError> {
        let name = metadata.name.clone();
        
        if self.plugins.contains_key(&name) {
            return Err(PluginError::InitializationFailed(
                format!("Plugin '{}' is already registered", name)
            ));
        }
        
        let entry = PluginEntry {
            metadata,
            capabilities,
            plugin: Arc::new(RwLock::new(plugin)),
            enabled: true,
        };
        
        self.plugins.insert(name, entry);
        Ok(())
    }
    
    /// Register a plugin with aliases
    pub fn register_plugin_with_aliases(
        &mut self,
        plugin: Box<dyn Plugin>,
        metadata: PluginMetadata,
        capabilities: PluginCapabilities,
        aliases: Vec<String>,
    ) -> Result<(), PluginError> {
        let name = metadata.name.clone();
        
        // Register the main plugin
        self.register_plugin(plugin, metadata, capabilities)?;
        
        // Register aliases
        for alias in aliases {
            if self.aliases.contains_key(&alias) || self.plugins.contains_key(&alias) {
                return Err(PluginError::InitializationFailed(
                    format!("Alias '{}' is already in use", alias)
                ));
            }
            self.aliases.insert(alias, name.clone());
        }
        
        Ok(())
    }
    
    /// Get a plugin by name or alias
    pub fn get_plugin(&self, name: &str) -> Option<&PluginEntry> {
        // Try direct lookup first
        if let Some(entry) = self.plugins.get(name) {
            return Some(entry);
        }
        
        // Try alias lookup
        if let Some(real_name) = self.aliases.get(name) {
            return self.plugins.get(real_name);
        }
        
        None
    }
    
    /// Execute a method on a plugin
    pub fn execute_plugin_method(
        &self,
        plugin_name: &str,
        method: &str,
        args: &HashMap<String, JsonValue>,
    ) -> RuntimeResult<JsonValue> {
        let entry = self.get_plugin(plugin_name)
            .ok_or_else(|| sigmos_runtime::RuntimeError::Plugin(
                format!("Plugin '{}' not found", plugin_name)
            ))?;
        
        if !entry.enabled {
            return Err(sigmos_runtime::RuntimeError::Plugin(
                format!("Plugin '{}' is disabled", plugin_name)
            ));
        }
        
        let plugin = entry.plugin.read().map_err(|_| {
            sigmos_runtime::RuntimeError::Plugin("Failed to acquire plugin lock".to_string())
        })?;
        
        plugin.execute(method, args)
    }
    
    /// Enable a plugin
    pub fn enable_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        let real_name = self.resolve_name(name);
        
        if let Some(entry) = self.plugins.get_mut(&real_name) {
            entry.enabled = true;
            Ok(())
        } else {
            Err(PluginError::ExecutionFailed(
                format!("Plugin '{}' not found", name)
            ))
        }
    }
    
    /// Disable a plugin
    pub fn disable_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        let real_name = self.resolve_name(name);
        
        if let Some(entry) = self.plugins.get_mut(&real_name) {
            entry.enabled = false;
            Ok(())
        } else {
            Err(PluginError::ExecutionFailed(
                format!("Plugin '{}' not found", name)
            ))
        }
    }
    
    /// Unregister a plugin
    pub fn unregister_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        let real_name = self.resolve_name(name);
        
        if self.plugins.remove(&real_name).is_some() {
            // Remove any aliases pointing to this plugin
            self.aliases.retain(|_, plugin_name| plugin_name != &real_name);
            Ok(())
        } else {
            Err(PluginError::ExecutionFailed(
                format!("Plugin '{}' not found", name)
            ))
        }
    }
    
    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<PluginRegistrationInfo> {
        self.plugins
            .iter()
            .map(|(name, entry)| {
                let aliases = self.aliases
                    .iter()
                    .filter_map(|(alias, plugin_name)| {
                        if plugin_name == name {
                            Some(alias.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                
                PluginRegistrationInfo {
                    name: entry.metadata.name.clone(),
                    version: entry.metadata.version.clone(),
                    description: entry.metadata.description.clone(),
                    author: entry.metadata.author.clone(),
                    methods: entry.metadata.methods.clone(),
                    aliases,
                }
            })
            .collect()
    }
    
    /// Get plugin metadata
    pub fn get_plugin_metadata(&self, name: &str) -> Option<&PluginMetadata> {
        self.get_plugin(name).map(|entry| &entry.metadata)
    }
    
    /// Get plugin capabilities
    pub fn get_plugin_capabilities(&self, name: &str) -> Option<&PluginCapabilities> {
        self.get_plugin(name).map(|entry| &entry.capabilities)
    }
    
    /// Check if a plugin is enabled
    pub fn is_plugin_enabled(&self, name: &str) -> bool {
        self.get_plugin(name)
            .map(|entry| entry.enabled)
            .unwrap_or(false)
    }
    
    /// Get the number of registered plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
    
    /// Check if a plugin exists
    pub fn has_plugin(&self, name: &str) -> bool {
        self.get_plugin(name).is_some()
    }
    
    /// Resolve plugin name (handles aliases)
    fn resolve_name(&self, name: &str) -> String {
        self.aliases.get(name).cloned().unwrap_or_else(|| name.to_string())
    }
    
    /// Initialize all registered plugins
    pub fn initialize_all(&self) -> Result<Vec<String>, PluginError> {
        let mut failed_plugins = Vec::new();
        
        for (name, entry) in &self.plugins {
            if entry.enabled {
                if let Ok(mut plugin) = entry.plugin.write() {
                    if let Err(e) = plugin.initialize() {
                        failed_plugins.push(format!("{}: {}", name, e));
                    }
                } else {
                    failed_plugins.push(format!("{}: Failed to acquire write lock", name));
                }
            }
        }
        
        if failed_plugins.is_empty() {
            Ok(Vec::new())
        } else {
            Err(PluginError::InitializationFailed(
                format!("Failed to initialize plugins: {}", failed_plugins.join(", "))
            ))
        }
    }
    
    /// Get plugins by capability
    pub fn get_plugins_by_capability(&self, check: impl Fn(&PluginCapabilities) -> bool) -> Vec<String> {
        self.plugins
            .iter()
            .filter_map(|(name, entry)| {
                if entry.enabled && check(&entry.capabilities) {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sigmos_runtime::{Plugin, RuntimeResult};
    
    #[derive(Debug)]
    struct MockPlugin {
        name: String,
        initialized: bool,
    }
    
    impl MockPlugin {
        fn new(name: String) -> Self {
            Self { name, initialized: false }
        }
    }
    
    impl Plugin for MockPlugin {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn initialize(&mut self) -> RuntimeResult<()> {
            self.initialized = true;
            Ok(())
        }
        
        fn execute(
            &self,
            method: &str,
            _args: &HashMap<String, JsonValue>,
        ) -> RuntimeResult<JsonValue> {
            Ok(JsonValue::String(format!("{}:{}", self.name, method)))
        }
    }

    #[test]
    fn test_plugin_registry_basic() {
        let mut registry = PluginRegistry::new();
        assert_eq!(registry.plugin_count(), 0);
        
        let plugin = Box::new(MockPlugin::new("test".to_string()));
        let metadata = PluginMetadata {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            methods: vec!["test_method".to_string()],
        };
        let capabilities = PluginCapabilities {
            supports_async: false,
            supports_streaming: false,
            requires_network: false,
            requires_auth: false,
        };
        
        assert!(registry.register_plugin(plugin, metadata, capabilities).is_ok());
        assert_eq!(registry.plugin_count(), 1);
        assert!(registry.has_plugin("test"));
    }

    #[test]
    fn test_plugin_registry_aliases() {
        let mut registry = PluginRegistry::new();
        
        let plugin = Box::new(MockPlugin::new("test".to_string()));
        let metadata = PluginMetadata {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            methods: vec!["test_method".to_string()],
        };
        let capabilities = PluginCapabilities {
            supports_async: false,
            supports_streaming: false,
            requires_network: false,
            requires_auth: false,
        };
        let aliases = vec!["t".to_string(), "testing".to_string()];
        
        assert!(registry.register_plugin_with_aliases(plugin, metadata, capabilities, aliases).is_ok());
        
        assert!(registry.has_plugin("test"));
        assert!(registry.has_plugin("t"));
        assert!(registry.has_plugin("testing"));
    }

    #[test]
    fn test_plugin_execution() {
        let mut registry = PluginRegistry::new();
        
        let plugin = Box::new(MockPlugin::new("test".to_string()));
        let metadata = PluginMetadata {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            methods: vec!["test_method".to_string()],
        };
        let capabilities = PluginCapabilities {
            supports_async: false,
            supports_streaming: false,
            requires_network: false,
            requires_auth: false,
        };
        
        registry.register_plugin(plugin, metadata, capabilities).unwrap();
        
        let args = HashMap::new();
        let result = registry.execute_plugin_method("test", "test_method", &args);
        assert!(result.is_ok());
        
        let value = result.unwrap();
        assert_eq!(value, JsonValue::String("test:test_method".to_string()));
    }
}

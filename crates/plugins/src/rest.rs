//! REST API Plugin
//!
//! This plugin provides HTTP/REST API integration capabilities for SIGMOS,
//! enabling interaction with web services, APIs, and HTTP endpoints.

use crate::{ConfigurablePlugin, PluginConfig, PluginError, PluginCapabilities, PluginMetadata};
use sigmos_runtime::{Plugin, RuntimeResult, RuntimeError};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// HTTP method enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::HEAD => write!(f, "HEAD"),
            HttpMethod::OPTIONS => write!(f, "OPTIONS"),
        }
    }
}

/// REST plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestConfig {
    pub name: String,
    pub base_url: String,
    pub default_headers: HashMap<String, String>,
    pub timeout_seconds: u64,
    pub max_redirects: u32,
    pub verify_ssl: bool,
    pub auth_token: Option<String>,
    pub user_agent: String,
}

impl PluginConfig for RestConfig {
    fn validate(&self) -> Result<(), PluginError> {
        if self.name.is_empty() {
            return Err(PluginError::InvalidConfiguration("name cannot be empty".to_string()));
        }
        if self.base_url.is_empty() {
            return Err(PluginError::InvalidConfiguration("base_url cannot be empty".to_string()));
        }
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(PluginError::InvalidConfiguration("base_url must start with http:// or https://".to_string()));
        }
        if self.timeout_seconds == 0 {
            return Err(PluginError::InvalidConfiguration("timeout_seconds must be greater than 0".to_string()));
        }
        Ok(())
    }
    
    fn plugin_name(&self) -> &str {
        &self.name
    }
}

impl Default for RestConfig {
    fn default() -> Self {
        let mut default_headers = HashMap::new();
        default_headers.insert("Content-Type".to_string(), "application/json".to_string());
        default_headers.insert("Accept".to_string(), "application/json".to_string());
        
        Self {
            name: "rest".to_string(),
            base_url: "https://api.example.com".to_string(),
            default_headers,
            timeout_seconds: 30,
            max_redirects: 5,
            verify_ssl: true,
            auth_token: None,
            user_agent: "SIGMOS-REST-Plugin/1.0".to_string(),
        }
    }
}

/// REST Plugin for HTTP/API integration
#[derive(Debug)]
pub struct RestPlugin {
    config: RestConfig,
    initialized: bool,
}

impl ConfigurablePlugin for RestPlugin {
    type Config = RestConfig;
    
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

impl Plugin for RestPlugin {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn initialize(&mut self) -> RuntimeResult<()> {
        // Initialize HTTP client
        // For now, this is a placeholder - would normally set up HTTP client with config
        self.initialized = true;
        Ok(())
    }
    
    fn execute(
        &self,
        method: &str,
        args: &HashMap<String, JsonValue>,
    ) -> RuntimeResult<JsonValue> {
        if !self.initialized {
            return Err(RuntimeError::Plugin("REST plugin not initialized".to_string()));
        }
        
        match method {
            "get" => self.http_request(HttpMethod::GET, args),
            "post" => self.http_request(HttpMethod::POST, args),
            "put" => self.http_request(HttpMethod::PUT, args),
            "delete" => self.http_request(HttpMethod::DELETE, args),
            "patch" => self.http_request(HttpMethod::PATCH, args),
            "head" => self.http_request(HttpMethod::HEAD, args),
            "options" => self.http_request(HttpMethod::OPTIONS, args),
            "request" => self.generic_request(args),
            _ => Err(RuntimeError::Plugin(format!("Unknown REST method: {}", method))),
        }
    }
}

impl RestPlugin {
    /// Generic HTTP request method
    fn http_request(&self, method: HttpMethod, args: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let url = if path.is_empty() {
            self.config.base_url.clone()
        } else {
            format!("{}/{}", self.config.base_url.trim_end_matches('/'), path.trim_start_matches('/'))
        };
        
        // Placeholder implementation - would normally make actual HTTP request
        let response = self.mock_http_response(&method, &url, args);
        Ok(response)
    }
    
    /// Generic request with custom method
    fn generic_request(&self, args: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue> {
        let method_str = args.get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RuntimeError::Plugin("Missing 'method' argument".to_string()))?;
        
        let method = match method_str.to_uppercase().as_str() {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "PATCH" => HttpMethod::PATCH,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            _ => return Err(RuntimeError::Plugin(format!("Unsupported HTTP method: {}", method_str))),
        };
        
        self.http_request(method, args)
    }
    
    /// Mock HTTP response for testing/placeholder
    fn mock_http_response(&self, method: &HttpMethod, url: &str, args: &HashMap<String, JsonValue>) -> JsonValue {
        let mut response = serde_json::Map::new();
        
        response.insert("status".to_string(), JsonValue::Number(serde_json::Number::from(200)));
        response.insert("method".to_string(), JsonValue::String(method.to_string()));
        response.insert("url".to_string(), JsonValue::String(url.to_string()));
        response.insert("timestamp".to_string(), JsonValue::String(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string()
        ));
        
        // Mock response body based on method
        let body = match method {
            HttpMethod::GET => {
                let mut data = serde_json::Map::new();
                data.insert("message".to_string(), JsonValue::String("GET request successful".to_string()));
                data.insert("data".to_string(), JsonValue::Array(vec![
                    JsonValue::String("item1".to_string()),
                    JsonValue::String("item2".to_string()),
                    JsonValue::String("item3".to_string()),
                ]));
                JsonValue::Object(data)
            },
            HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH => {
                let mut data = serde_json::Map::new();
                data.insert("message".to_string(), JsonValue::String(format!("{} request successful", method)));
                if let Some(body) = args.get("body") {
                    data.insert("received_data".to_string(), body.clone());
                }
                data.insert("id".to_string(), JsonValue::Number(serde_json::Number::from(12345)));
                JsonValue::Object(data)
            },
            HttpMethod::DELETE => {
                let mut data = serde_json::Map::new();
                data.insert("message".to_string(), JsonValue::String("DELETE request successful".to_string()));
                data.insert("deleted".to_string(), JsonValue::Bool(true));
                JsonValue::Object(data)
            },
            HttpMethod::HEAD | HttpMethod::OPTIONS => {
                JsonValue::Null
            },
        };
        
        response.insert("body".to_string(), body);
        response.insert("headers".to_string(), JsonValue::Object({
            let mut headers = serde_json::Map::new();
            headers.insert("content-type".to_string(), JsonValue::String("application/json".to_string()));
            headers.insert("server".to_string(), JsonValue::String("SIGMOS-Mock-Server".to_string()));
            headers
        }));
        
        JsonValue::Object(response)
    }
    
    /// Get plugin metadata
    pub fn metadata() -> PluginMetadata {
        PluginMetadata {
            name: "rest".to_string(),
            version: "1.0.0".to_string(),
            description: "HTTP/REST API integration plugin".to_string(),
            author: "SIGMOS Team".to_string(),
            methods: vec![
                "get".to_string(),
                "post".to_string(),
                "put".to_string(),
                "delete".to_string(),
                "patch".to_string(),
                "head".to_string(),
                "options".to_string(),
                "request".to_string(),
            ],
        }
    }
    
    /// Get plugin capabilities
    pub fn capabilities() -> PluginCapabilities {
        PluginCapabilities {
            supports_async: true,
            supports_streaming: false,
            requires_network: true,
            requires_auth: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_config_validation() {
        let mut config = RestConfig::default();
        assert!(config.validate().is_ok());
        
        config.name = "".to_string();
        assert!(config.validate().is_err());
        
        config.name = "test".to_string();
        config.base_url = "invalid-url".to_string();
        assert!(config.validate().is_err());
        
        config.base_url = "https://api.example.com".to_string();
        config.timeout_seconds = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_rest_plugin_creation() {
        let config = RestConfig::default();
        let plugin = RestPlugin::new(config);
        assert!(plugin.is_ok());
    }

    #[test]
    fn test_rest_plugin_methods() {
        let config = RestConfig::default();
        let mut plugin = RestPlugin::new(config).unwrap();
        assert!(plugin.initialize().is_ok());
        
        let mut args = HashMap::new();
        args.insert("path".to_string(), JsonValue::String("/users".to_string()));
        
        let result = plugin.execute("get", &args);
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.get("status").is_some());
        assert!(response.get("method").is_some());
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::GET.to_string(), "GET");
        assert_eq!(HttpMethod::POST.to_string(), "POST");
        assert_eq!(HttpMethod::DELETE.to_string(), "DELETE");
    }
}

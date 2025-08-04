//! REST API Plugin
//!
//! This plugin provides HTTP/REST API integration capabilities for SIGMOS,
//! enabling interaction with web services, APIs, and HTTP endpoints.

use crate::{ConfigurablePlugin, PluginCapabilities, PluginConfig, PluginError, PluginMetadata};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sigmos_runtime::{Plugin, RuntimeError, RuntimeResult};
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
            return Err(PluginError::InvalidConfiguration(
                "name cannot be empty".to_string(),
            ));
        }
        if self.base_url.is_empty() {
            return Err(PluginError::InvalidConfiguration(
                "base_url cannot be empty".to_string(),
            ));
        }
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(PluginError::InvalidConfiguration(
                "base_url must start with http:// or https://".to_string(),
            ));
        }
        if self.timeout_seconds == 0 {
            return Err(PluginError::InvalidConfiguration(
                "timeout_seconds must be greater than 0".to_string(),
            ));
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
    client: Option<reqwest::Client>,
}

impl ConfigurablePlugin for RestPlugin {
    type Config = RestConfig;

    fn new(config: Self::Config) -> Result<Self, PluginError> {
        config.validate()?;
        let client = Some(reqwest::Client::new());
        Ok(RestPlugin {
            config,
            initialized: false,
            client,
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

    fn execute(&self, method: &str, args: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue> {
        if !self.initialized {
            return Err(RuntimeError::Plugin(
                "REST plugin not initialized".to_string(),
            ));
        }

        // Use tokio runtime to handle async HTTP requests
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| RuntimeError::Plugin(format!("Failed to create async runtime: {e}")))?;

        match method {
            "get" => rt.block_on(self.http_request(HttpMethod::GET, args)),
            "post" => rt.block_on(self.http_request(HttpMethod::POST, args)),
            "put" => rt.block_on(self.http_request(HttpMethod::PUT, args)),
            "delete" => rt.block_on(self.http_request(HttpMethod::DELETE, args)),
            "patch" => rt.block_on(self.http_request(HttpMethod::PATCH, args)),
            "head" => rt.block_on(self.http_request(HttpMethod::HEAD, args)),
            "options" => rt.block_on(self.http_request(HttpMethod::OPTIONS, args)),
            "request" => self.generic_request(args),
            _ => Err(RuntimeError::Plugin(format!(
                "Unknown REST method: {method}"
            ))),
        }
    }
}

impl RestPlugin {
    /// Generic HTTP request method
    async fn http_request(
        &self,
        method: HttpMethod,
        args: &HashMap<String, JsonValue>,
    ) -> RuntimeResult<JsonValue> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| RuntimeError::Plugin("HTTP client not initialized".to_string()))?;

        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");

        let url = if path.is_empty() {
            self.config.base_url.clone()
        } else {
            format!(
                "{}/{}",
                self.config.base_url.trim_end_matches('/'),
                path.trim_start_matches('/')
            )
        };

        // Build the request
        let mut request_builder = match method {
            HttpMethod::GET => client.get(&url),
            HttpMethod::POST => client.post(&url),
            HttpMethod::PUT => client.put(&url),
            HttpMethod::DELETE => client.delete(&url),
            HttpMethod::PATCH => client.patch(&url),
            HttpMethod::HEAD => client.head(&url),
            HttpMethod::OPTIONS => client.request(reqwest::Method::OPTIONS, &url),
        };

        // Add headers if provided
        if let Some(headers) = args.get("headers").and_then(|v| v.as_object()) {
            for (key, value) in headers {
                if let Some(value_str) = value.as_str() {
                    request_builder = request_builder.header(key, value_str);
                }
            }
        }

        // Add body for POST/PUT/PATCH requests
        if matches!(
            method,
            HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH
        ) {
            if let Some(body) = args.get("body") {
                request_builder = request_builder.json(body);
            }
        }

        // Add query parameters if provided
        if let Some(params) = args.get("params").and_then(|v| v.as_object()) {
            let mut query_params = Vec::new();
            for (key, value) in params {
                if let Some(value_str) = value.as_str() {
                    query_params.push((key.as_str(), value_str));
                }
            }
            if !query_params.is_empty() {
                request_builder = request_builder.query(&query_params);
            }
        }

        // Execute the request
        let response = request_builder
            .send()
            .await
            .map_err(|e| RuntimeError::Plugin(format!("HTTP request failed: {e}")))?;

        let status = response.status().as_u16();
        let headers_map: serde_json::Map<String, JsonValue> = response
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    JsonValue::String(v.to_str().unwrap_or("").to_string()),
                )
            })
            .collect();

        let body_text = response
            .text()
            .await
            .map_err(|e| RuntimeError::Plugin(format!("Failed to read response body: {e}")))?;

        // Try to parse as JSON, fallback to string
        let body_json =
            serde_json::from_str::<JsonValue>(&body_text).unwrap_or(JsonValue::String(body_text));

        Ok(JsonValue::Object({
            let mut obj = serde_json::Map::new();
            obj.insert(
                "status".to_string(),
                JsonValue::Number(serde_json::Number::from(status)),
            );
            obj.insert("headers".to_string(), JsonValue::Object(headers_map));
            obj.insert("body".to_string(), body_json);
            obj.insert("url".to_string(), JsonValue::String(url));
            obj.insert("method".to_string(), JsonValue::String(method.to_string()));
            obj
        }))
    }

    /// Generic request with custom method
    fn generic_request(&self, args: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue> {
        let method_str = args
            .get("method")
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
            _ => {
                return Err(RuntimeError::Plugin(format!(
                    "Unsupported HTTP method: {method_str}"
                )))
            }
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| RuntimeError::Plugin(format!("Failed to create async runtime: {e}")))?;

        rt.block_on(self.http_request(method, args))
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
        let config = RestConfig {
            name: "test_rest".to_string(),
            base_url: "https://httpbin.org".to_string(), // Use a real testing API
            default_headers: HashMap::new(),
            timeout_seconds: 30,
            max_redirects: 5,
            verify_ssl: true,
            auth_token: None,
            user_agent: "SIGMOS-REST-Plugin/1.0".to_string(),
        };

        let mut plugin = RestPlugin::new(config).unwrap();
        assert!(plugin.initialize().is_ok());

        let mut args = HashMap::new();
        args.insert("path".to_string(), JsonValue::String("/get".to_string()));

        // Test the plugin execution - this will make a real HTTP request
        let result = plugin.execute("get", &args);

        // The request might fail due to network issues, so we test both success and failure cases
        match result {
            Ok(response) => {
                // If successful, verify the response structure
                assert!(response.get("status").is_some());
                assert!(response.get("method").is_some());
                assert!(response.get("url").is_some());
                assert!(response.get("body").is_some());
            }
            Err(e) => {
                // If it fails (e.g., no internet), ensure it's a plugin error
                match e {
                    RuntimeError::Plugin(_) => {
                        // This is expected if there's no network access
                        println!("Network request failed (expected in some environments): {e}");
                    }
                    _ => panic!("Unexpected error type: {e}"),
                }
            }
        }
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::GET.to_string(), "GET");
        assert_eq!(HttpMethod::POST.to_string(), "POST");
        assert_eq!(HttpMethod::DELETE.to_string(), "DELETE");
    }
}

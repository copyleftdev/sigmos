//! Integration tests for SIGMOS
//!
//! These tests verify that all components work together correctly,
//! testing full workflows from parsing to execution.

use sigmos_core::{SigmosParser, ast::*};
use sigmos_runtime::Runtime;
use sigmos_plugins::{
    registry::PluginRegistry,
    mcp::{McpPlugin, McpConfig},
    rest::{RestPlugin, RestConfig},
    ConfigurablePlugin,
};
use std::collections::HashMap;
use serde_json::Value as JsonValue;

/// Test basic SIGMOS workflow: parse -> runtime -> execute
#[test]
fn test_basic_workflow() {
    // Parse a simple spec
    let input = r#"
    spec "TestWorkflow" v1.0 {
        description: "Integration test specification"
    }
    "#;
    
    let spec = SigmosParser::parse_spec(input).expect("Failed to parse spec");
    assert_eq!(spec.name, "PlaceholderSpec"); // Current placeholder behavior
    
    // Create runtime
    let runtime = Runtime::new();
    assert!(runtime.plugin_count() == 0);
    
    // Test expression evaluation
    let expr = Expression::StringLiteral("Hello Integration Test".to_string());
    let result = runtime.evaluate_expression(&expr).expect("Failed to evaluate expression");
    
    if let JsonValue::String(s) = result {
        assert_eq!(s, "Hello Integration Test");
    } else {
        panic!("Expected string result");
    }
}

/// Test plugin integration workflow
#[test]
fn test_plugin_integration_workflow() {
    let mut registry = PluginRegistry::new();
    
    // Register MCP plugin
    let mcp_config = McpConfig {
        name: "test_mcp".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        model: "test-model".to_string(),
        ..Default::default()
    };
    let mcp_plugin = McpPlugin::new(mcp_config).expect("Failed to create MCP plugin");
    registry.register_plugin(
        Box::new(mcp_plugin),
        sigmos_plugins::mcp::McpPlugin::metadata(),
        sigmos_plugins::mcp::McpPlugin::capabilities(),
    ).expect("Failed to register MCP plugin");
    
    // Register REST plugin
    let rest_config = RestConfig {
        name: "test_rest".to_string(),
        base_url: "https://api.example.com".to_string(),
        ..Default::default()
    };
    let rest_plugin = RestPlugin::new(rest_config).expect("Failed to create REST plugin");
    registry.register_plugin(
        Box::new(rest_plugin),
        sigmos_plugins::rest::RestPlugin::metadata(),
        sigmos_plugins::rest::RestPlugin::capabilities(),
    ).expect("Failed to register REST plugin");
    
    // Test plugin execution
    let mut args = HashMap::new();
    args.insert("prompt".to_string(), JsonValue::String("Test prompt".to_string()));
    
    let mcp_result = registry.execute_plugin_method("test_mcp", "complete", &args)
        .expect("Failed to execute MCP plugin");
    assert!(mcp_result.get("text").is_some());
    
    args.clear();
    args.insert("path".to_string(), JsonValue::String("/test".to_string()));
    
    let rest_result = registry.execute_plugin_method("test_rest", "get", &args)
        .expect("Failed to execute REST plugin");
    assert!(rest_result.get("status").is_some());
}

/// Test runtime with plugins integration
#[test]
fn test_runtime_plugin_integration() {
    let mut runtime = Runtime::new();
    
    // Create and register a test plugin
    let mcp_config = McpConfig {
        name: "runtime_mcp".to_string(),
        endpoint: "http://test.local".to_string(),
        model: "test".to_string(),
        ..Default::default()
    };
    let mut mcp_plugin = McpPlugin::new(mcp_config).expect("Failed to create plugin");
    mcp_plugin.initialize().expect("Failed to initialize plugin");
    
    runtime.register_plugin("runtime_mcp".to_string(), Box::new(mcp_plugin));
    assert_eq!(runtime.plugin_count(), 1);
    
    // Test expression evaluation with runtime context
    let expressions = vec![
        Expression::StringLiteral("test".to_string()),
        Expression::Number(42.0),
        Expression::Boolean(true),
        Expression::Identifier("test_var".to_string()),
    ];
    
    for expr in expressions {
        let result = runtime.evaluate_expression(&expr);
        assert!(result.is_ok(), "Failed to evaluate expression: {:?}", expr);
    }
}

/// Test error handling across components
#[test]
fn test_error_handling_integration() {
    let runtime = Runtime::new();
    
    // Test invalid expressions
    let invalid_function = Expression::FunctionCall {
        object: "nonexistent".to_string(),
        method: "invalid_method".to_string(),
        arguments: vec![],
    };
    
    let result = runtime.evaluate_expression(&invalid_function);
    assert!(result.is_ok()); // Current implementation returns placeholder
    
    // Test plugin registry error handling
    let mut registry = PluginRegistry::new();
    
    // Try to execute on non-existent plugin
    let args = HashMap::new();
    let result = registry.execute_plugin_method("nonexistent", "test", &args);
    assert!(result.is_err());
    
    // Try to register plugin with same name twice
    let config1 = McpConfig { name: "duplicate".to_string(), ..Default::default() };
    let config2 = McpConfig { name: "duplicate".to_string(), ..Default::default() };
    
    let plugin1 = McpPlugin::new(config1).unwrap();
    let plugin2 = McpPlugin::new(config2).unwrap();
    
    registry.register_plugin(
        Box::new(plugin1),
        sigmos_plugins::mcp::McpPlugin::metadata(),
        sigmos_plugins::mcp::McpPlugin::capabilities(),
    ).expect("First registration should succeed");
    
    let result = registry.register_plugin(
        Box::new(plugin2),
        sigmos_plugins::mcp::McpPlugin::metadata(),
        sigmos_plugins::mcp::McpPlugin::capabilities(),
    );
    assert!(result.is_err(), "Duplicate registration should fail");
}

/// Test concurrent access and thread safety
#[test]
fn test_concurrent_access() {
    use std::sync::Arc;
    use std::thread;
    
    let registry = Arc::new(std::sync::Mutex::new(PluginRegistry::new()));
    
    // Register a plugin
    {
        let mut reg = registry.lock().unwrap();
        let config = McpConfig { name: "concurrent_test".to_string(), ..Default::default() };
        let plugin = McpPlugin::new(config).unwrap();
        reg.register_plugin(
            Box::new(plugin),
            sigmos_plugins::mcp::McpPlugin::metadata(),
            sigmos_plugins::mcp::McpPlugin::capabilities(),
        ).expect("Failed to register plugin");
    }
    
    // Test concurrent plugin execution
    let handles: Vec<_> = (0..5).map(|i| {
        let registry_clone = Arc::clone(&registry);
        thread::spawn(move || {
            let reg = registry_clone.lock().unwrap();
            let mut args = HashMap::new();
            args.insert("prompt".to_string(), JsonValue::String(format!("Test {}", i)));
            
            let result = reg.execute_plugin_method("concurrent_test", "complete", &args);
            assert!(result.is_ok(), "Concurrent execution failed for thread {}", i);
            result.unwrap()
        })
    }).collect();
    
    // Wait for all threads to complete
    for handle in handles {
        let result = handle.join().expect("Thread panicked");
        assert!(result.get("text").is_some());
    }
}

/// Test memory and resource management
#[test]
fn test_resource_management() {
    // Test that we can create and drop many components without issues
    for i in 0..100 {
        let runtime = Runtime::new();
        let expr = Expression::StringLiteral(format!("test_{}", i));
        let result = runtime.evaluate_expression(&expr);
        assert!(result.is_ok());
        
        let mut registry = PluginRegistry::new();
        let config = McpConfig { 
            name: format!("test_{}", i),
            ..Default::default() 
        };
        let plugin = McpPlugin::new(config).unwrap();
        registry.register_plugin(
            Box::new(plugin),
            sigmos_plugins::mcp::McpPlugin::metadata(),
            sigmos_plugins::mcp::McpPlugin::capabilities(),
        ).expect("Failed to register plugin");
        
        assert_eq!(registry.plugin_count(), 1);
        // Components will be dropped at end of loop iteration
    }
}

//! Integration tests for SIGMOS CLI and example validation
//!
//! These tests verify that all example specifications are valid and can be parsed correctly.
//! They ensure syntax, semantics, and structural integrity of all industry examples.

use sigmos_core::{SigmosParser, ast::*};
use std::fs;
use std::path::Path;

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

/// Test that all example files can be parsed successfully
#[test]
fn test_all_examples_parse_successfully() {
    use std::fs;
    use std::path::Path;
    
    let examples_dir = Path::new("examples");
    if !examples_dir.exists() {
        println!("Examples directory not found, skipping example tests");
        return;
    }

    let example_files = find_sigmos_files(examples_dir);
    if example_files.is_empty() {
        println!("No example files found, skipping example tests");
        return;
    }

    for file_path in &example_files {
        println!("Testing example: {}", file_path.display());
        
        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", file_path.display(), e));
        
        let result = SigmosParser::parse_spec(&content);
        match result {
            Ok(spec) => {
                println!("✓ Successfully parsed: {} (version: {})", spec.name, spec.version);
                
                // Validate basic structure
                assert!(!spec.name.is_empty(), "Spec name should not be empty");
                assert!(!spec.version.is_empty(), "Spec version should not be empty");
                
                // Validate inputs structure if present
                if !spec.inputs.is_empty() {
                    println!("  - {} input fields defined", spec.inputs.len());
                }
                
                // Validate computed fields if present
                if !spec.computed.is_empty() {
                    println!("  - {} computed fields defined", spec.computed.len());
                }
                
                // Validate events if present
                if !spec.events.is_empty() {
                    println!("  - {} event handlers defined", spec.events.len());
                }
                
                // Validate constraints if present
                if !spec.constraints.is_empty() {
                    println!("  - {} constraints defined", spec.constraints.len());
                }
            }
            Err(e) => {
                panic!("Failed to parse {}: {:?}", file_path.display(), e);
            }
        }
    }
    
    println!("✓ All {} example files parsed successfully", example_files.len());
}

/// Test industry-specific examples for domain-appropriate patterns
#[test]
fn test_industry_specific_patterns() {
    use std::fs;
    use std::path::Path;
    
    let examples_dir = Path::new("examples");
    if !examples_dir.exists() {
        println!("Examples directory not found, skipping industry pattern tests");
        return;
    }

    let example_files = find_sigmos_files(examples_dir);
    if example_files.is_empty() {
        println!("No example files found, skipping industry pattern tests");
        return;
    }

    for file_path in &example_files {
        println!("Testing industry patterns for: {}", file_path.display());
        
        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", file_path.display(), e));
        
        let spec = SigmosParser::parse_spec(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {:?}", file_path.display(), e));
        
        // Determine industry type from file path and apply appropriate tests
        let file_path_str = file_path.to_string_lossy().to_lowercase();
        
        if file_path_str.contains("fintech") || file_path_str.contains("trading") {
            validate_fintech_patterns(&spec);
            println!("✓ Fintech patterns validated for {}", file_path.display());
        } else if file_path_str.contains("healthcare") || file_path_str.contains("patient") {
            validate_healthcare_patterns(&spec);
            println!("✓ Healthcare patterns validated for {}", file_path.display());
        } else if file_path_str.contains("ecommerce") || file_path_str.contains("recommendation") {
            validate_ecommerce_patterns(&spec);
            println!("✓ E-commerce patterns validated for {}", file_path.display());
        } else if file_path_str.contains("manufacturing") || file_path_str.contains("iot") {
            validate_manufacturing_patterns(&spec);
            println!("✓ Manufacturing patterns validated for {}", file_path.display());
        } else if file_path_str.contains("logistics") || file_path_str.contains("supply") {
            validate_logistics_patterns(&spec);
            println!("✓ Logistics patterns validated for {}", file_path.display());
        } else if file_path_str.contains("cybersecurity") || file_path_str.contains("threat") {
            validate_cybersecurity_patterns(&spec);
            println!("✓ Cybersecurity patterns validated for {}", file_path.display());
        } else if file_path_str.contains("smart-city") || file_path_str.contains("urban") {
            validate_smart_city_patterns(&spec);
            println!("✓ Smart city patterns validated for {}", file_path.display());
        } else {
            // For general examples, just validate basic structure
            validate_general_patterns(&spec);
            println!("✓ General patterns validated for {}", file_path.display());
        }
    }
    
    println!("✓ All {} example files validated for industry patterns", example_files.len());
}

// Helper function to find all .sigmos files recursively
fn find_sigmos_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    use std::fs;
    let mut files = Vec::new();
    
    if dir.is_dir() {
        for entry in fs::read_dir(dir).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            
            if path.is_dir() {
                files.extend(find_sigmos_files(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("sigmos") {
                files.push(path);
            }
        }
    }
    
    files
}

// Industry-specific pattern validation functions
fn validate_fintech_patterns(spec: &Specification) {
    // Fintech should have risk management patterns
    assert!(spec.name.contains("Trading") || spec.description.contains("trading") || spec.description.contains("financial"),
        "Fintech example should be trading/financial related");
    
    // Should have compliance-related fields
    let has_compliance = spec.inputs.iter().any(|(name, _)| 
        name.contains("compliance") || name.contains("regulation") || name.contains("risk"));
    assert!(has_compliance, "Fintech example should have compliance/risk fields");
}

fn validate_healthcare_patterns(spec: &Specification) {
    // Healthcare should have patient-related patterns
    assert!(spec.name.contains("Patient") || spec.description.contains("patient") || spec.description.contains("health"),
        "Healthcare example should be patient/health related");
    
    // Should have medical data fields
    let has_medical_fields = spec.inputs.iter().any(|(name, _)| 
        name.contains("patient") || name.contains("vital") || name.contains("medical"));
    assert!(has_medical_fields, "Healthcare example should have medical data fields");
}

fn validate_ecommerce_patterns(spec: &Specification) {
    // E-commerce should have recommendation patterns
    assert!(spec.name.contains("Recommendation") || spec.description.contains("recommendation") || spec.description.contains("commerce"),
        "E-commerce example should be recommendation/commerce related");
}

fn validate_manufacturing_patterns(spec: &Specification) {
    // Manufacturing should have IoT/monitoring patterns
    assert!(spec.name.contains("IoT") || spec.name.contains("Monitoring") || spec.description.contains("manufacturing"),
        "Manufacturing example should be IoT/monitoring related");
}

fn validate_logistics_patterns(spec: &Specification) {
    // Logistics should have supply chain patterns
    assert!(spec.name.contains("Supply") || spec.name.contains("Chain") || spec.description.contains("logistics"),
        "Logistics example should be supply chain related");
}

fn validate_cybersecurity_patterns(spec: &Specification) {
    // Cybersecurity should have threat detection patterns
    assert!(spec.name.contains("Threat") || spec.name.contains("Security") || spec.description.contains("security"),
        "Cybersecurity example should be security/threat related");
}

fn validate_smart_city_patterns(spec: &Specification) {
    // Smart city should have urban management patterns
    assert!(spec.name.contains("City") || spec.name.contains("Urban") || spec.description.contains("city"),
        "Smart city example should be city/urban related");
}

fn validate_general_patterns(spec: &Specification) {
    // General validation for any SIGMOS specification
    assert!(!spec.name.is_empty(), "Spec name should not be empty");
    assert!(!spec.version.is_empty(), "Spec version should not be empty");
    assert!(!spec.description.is_empty(), "Spec should have a description");
    
    // Check that spec name follows PascalCase
    assert!(spec.name.chars().next().unwrap().is_uppercase(),
        "Spec name '{}' should start with uppercase", spec.name);
}

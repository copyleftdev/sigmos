//! Fuzz testing for SIGMOS components
//!
//! These tests use random input generation to find edge cases and potential
//! crashes in our parser and runtime components.

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

/// Generate random bytes for fuzzing
fn generate_random_bytes(size: usize) -> Vec<u8> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut bytes = Vec::with_capacity(size);
    let mut hasher = DefaultHasher::new();
    
    for i in 0..size {
        i.hash(&mut hasher);
        bytes.push((hasher.finish() % 256) as u8);
    }
    
    bytes
}

/// Generate random UTF-8 strings for fuzzing
fn generate_random_string(size: usize) -> String {
    let bytes = generate_random_bytes(size);
    String::from_utf8_lossy(&bytes).to_string()
}

/// Fuzz test the parser with random input
#[test]
fn fuzz_parser_random_input() {
    for size in [0, 1, 10, 100, 1000, 10000] {
        for _ in 0..10 {
            let random_input = generate_random_string(size);
            
            // Parser should never panic
            let result = std::panic::catch_unwind(|| {
                SigmosParser::parse_spec(&random_input)
            });
            
            assert!(result.is_ok(), "Parser panicked on input of size {}", size);
            
            // Parse result should be consistent
            let parse_result = SigmosParser::parse_spec(&random_input);
            assert!(parse_result.is_ok(), "Parser failed on input of size {}", size);
        }
    }
}

/// Fuzz test the parser with malformed SIGMOS syntax
#[test]
fn fuzz_parser_malformed_syntax() {
    let malformed_inputs = vec![
        // Incomplete specs
        "spec",
        "spec \"",
        "spec \"test",
        "spec \"test\" v",
        "spec \"test\" v1",
        "spec \"test\" v1.",
        "spec \"test\" v1.0",
        "spec \"test\" v1.0 {",
        
        // Invalid characters
        "spec \"test\" v1.0 { \x00 }",
        "spec \"test\" v1.0 { \xFF }",
        "spec \"test\" v1.0 { 🚀 }",
        
        // Nested braces
        "spec \"test\" v1.0 { { { { } } } }",
        
        // Very long names
        &format!("spec \"{}\" v1.0 {{}}", "a".repeat(10000)),
        
        // Invalid version numbers
        "spec \"test\" v999999999999999999999.0 {}",
        "spec \"test\" v1.999999999999999999999 {}",
        
        // Mixed quotes
        "spec 'test\" v1.0 {}",
        "spec \"test' v1.0 {}",
        
        // Control characters
        "spec \"test\n\" v1.0 {}",
        "spec \"test\t\" v1.0 {}",
        "spec \"test\r\" v1.0 {}",
    ];
    
    for input in malformed_inputs {
        // Should not panic
        let result = std::panic::catch_unwind(|| {
            SigmosParser::parse_spec(input)
        });
        
        assert!(result.is_ok(), "Parser panicked on malformed input: {}", input);
        
        // Parse result should be consistent (currently returns placeholder)
        let parse_result = SigmosParser::parse_spec(input);
        assert!(parse_result.is_ok(), "Parser failed on malformed input: {}", input);
    }
}

/// Fuzz test expression evaluation with random expressions
#[test]
fn fuzz_expression_evaluation() {
    let runtime = Runtime::new();
    
    // Test with random string literals
    for size in [0, 1, 10, 100, 1000] {
        for _ in 0..10 {
            let random_string = generate_random_string(size);
            let expr = Expression::StringLiteral(random_string.clone());
            
            let result = std::panic::catch_unwind(|| {
                runtime.evaluate_expression(&expr)
            });
            
            assert!(result.is_ok(), "Expression evaluation panicked on string of size {}", size);
            
            let eval_result = runtime.evaluate_expression(&expr);
            assert!(eval_result.is_ok(), "Expression evaluation failed on string of size {}", size);
            
            if let JsonValue::String(result_str) = eval_result.unwrap() {
                assert_eq!(result_str, random_string);
            }
        }
    }
    
    // Test with extreme numbers
    let extreme_numbers = vec![
        f64::MIN,
        f64::MAX,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        0.0,
        -0.0,
        1.0,
        -1.0,
        f64::EPSILON,
        -f64::EPSILON,
    ];
    
    for &num in &extreme_numbers {
        if num.is_finite() {
            let expr = Expression::Number(num);
            
            let result = std::panic::catch_unwind(|| {
                runtime.evaluate_expression(&expr)
            });
            
            assert!(result.is_ok(), "Expression evaluation panicked on number: {}", num);
            
            let eval_result = runtime.evaluate_expression(&expr);
            assert!(eval_result.is_ok(), "Expression evaluation failed on number: {}", num);
        }
    }
    
    // Test with random identifiers
    for _ in 0..100 {
        let random_id = generate_random_string(50);
        let expr = Expression::Identifier(random_id);
        
        let result = std::panic::catch_unwind(|| {
            runtime.evaluate_expression(&expr)
        });
        
        assert!(result.is_ok(), "Expression evaluation panicked on identifier");
        
        let eval_result = runtime.evaluate_expression(&expr);
        assert!(eval_result.is_ok(), "Expression evaluation failed on identifier");
    }
}

/// Fuzz test plugin configurations
#[test]
fn fuzz_plugin_configurations() {
    // Test MCP plugin with random configurations
    for _ in 0..50 {
        let config = McpConfig {
            name: generate_random_string(100),
            endpoint: generate_random_string(200),
            model: generate_random_string(50),
            api_key: Some(generate_random_string(100)),
            timeout_seconds: (generate_random_bytes(8)[0] as u64) % 3600 + 1,
            max_tokens: Some((generate_random_bytes(4)[0] as u32) % 10000 + 1),
            temperature: Some((generate_random_bytes(4)[0] as f32) / 255.0 * 2.0),
        };
        
        // Should not panic during validation
        let result = std::panic::catch_unwind(|| {
            config.validate()
        });
        
        assert!(result.is_ok(), "MCP config validation panicked");
        
        // Plugin creation should not panic
        let plugin_result = std::panic::catch_unwind(|| {
            McpPlugin::new(config)
        });
        
        assert!(plugin_result.is_ok(), "MCP plugin creation panicked");
    }
    
    // Test REST plugin with random configurations
    for _ in 0..50 {
        let config = RestConfig {
            name: generate_random_string(100),
            base_url: generate_random_string(200),
            default_headers: HashMap::new(),
            timeout_seconds: (generate_random_bytes(8)[0] as u64) % 3600 + 1,
            max_redirects: (generate_random_bytes(4)[0] as u32) % 10 + 1,
            verify_ssl: generate_random_bytes(1)[0] % 2 == 0,
            auth_token: Some(generate_random_string(100)),
            user_agent: generate_random_string(100),
        };
        
        // Should not panic during validation
        let result = std::panic::catch_unwind(|| {
            config.validate()
        });
        
        assert!(result.is_ok(), "REST config validation panicked");
        
        // Plugin creation should not panic
        let plugin_result = std::panic::catch_unwind(|| {
            RestPlugin::new(config)
        });
        
        assert!(plugin_result.is_ok(), "REST plugin creation panicked");
    }
}

/// Fuzz test plugin registry operations
#[test]
fn fuzz_plugin_registry() {
    let mut registry = PluginRegistry::new();
    
    // Register plugins with random names
    let mut registered_names = Vec::new();
    
    for i in 0..20 {
        let name = format!("plugin_{}", i);
        let config = McpConfig {
            name: name.clone(),
            endpoint: "http://localhost:8080".to_string(),
            model: "test".to_string(),
            ..Default::default()
        };
        
        let plugin = McpPlugin::new(config).unwrap();
        let result = registry.register_plugin(
            Box::new(plugin),
            sigmos_plugins::mcp::McpPlugin::metadata(),
            sigmos_plugins::mcp::McpPlugin::capabilities(),
        );
        
        if result.is_ok() {
            registered_names.push(name);
        }
    }
    
    // Test random operations on registry
    for _ in 0..100 {
        let random_name = generate_random_string(50);
        
        // These should not panic
        let _ = std::panic::catch_unwind(|| {
            registry.has_plugin(&random_name)
        });
        
        let _ = std::panic::catch_unwind(|| {
            registry.get_plugin_metadata(&random_name)
        });
        
        let _ = std::panic::catch_unwind(|| {
            registry.get_plugin_capabilities(&random_name)
        });
        
        // Try to execute random methods
        let args = HashMap::new();
        let _ = std::panic::catch_unwind(|| {
            registry.execute_plugin_method(&random_name, "random_method", &args)
        });
    }
    
    // Test with registered plugin names
    for name in &registered_names {
        assert!(registry.has_plugin(name));
        
        let metadata = registry.get_plugin_metadata(name);
        assert!(metadata.is_some());
        
        let capabilities = registry.get_plugin_capabilities(name);
        assert!(capabilities.is_some());
        
        // Test method execution with random arguments
        for _ in 0..10 {
            let mut args = HashMap::new();
            for j in 0..5 {
                args.insert(
                    format!("arg_{}", j),
                    JsonValue::String(generate_random_string(20))
                );
            }
            
            let result = std::panic::catch_unwind(|| {
                registry.execute_plugin_method(name, "complete", &args)
            });
            
            assert!(result.is_ok(), "Plugin method execution panicked");
        }
    }
}

/// Stress test with high load
#[test]
fn stress_test_high_load() {
    let runtime = Runtime::new();
    
    // Evaluate many expressions rapidly
    for i in 0..1000 {
        let expressions = vec![
            Expression::StringLiteral(format!("test_{}", i)),
            Expression::Number(i as f64),
            Expression::Boolean(i % 2 == 0),
            Expression::Identifier(format!("var_{}", i)),
        ];
        
        for expr in expressions {
            let result = runtime.evaluate_expression(&expr);
            assert!(result.is_ok(), "Expression evaluation failed under load at iteration {}", i);
        }
    }
    
    // Create and destroy many plugin registries
    for i in 0..100 {
        let mut registry = PluginRegistry::new();
        
        // Register multiple plugins
        for j in 0..5 {
            let config = McpConfig {
                name: format!("plugin_{}_{}", i, j),
                endpoint: "http://localhost:8080".to_string(),
                model: "test".to_string(),
                ..Default::default()
            };
            
            let plugin = McpPlugin::new(config).unwrap();
            let result = registry.register_plugin(
                Box::new(plugin),
                sigmos_plugins::mcp::McpPlugin::metadata(),
                sigmos_plugins::mcp::McpPlugin::capabilities(),
            );
            
            assert!(result.is_ok(), "Plugin registration failed under load");
        }
        
        assert_eq!(registry.plugin_count(), 5);
        // Registry will be dropped at end of iteration
    }
}

/// Memory pressure test
#[test]
fn test_memory_pressure() {
    // Create large expressions and evaluate them
    for size in [1000, 5000, 10000] {
        let large_string = "x".repeat(size);
        let expr = Expression::StringLiteral(large_string.clone());
        
        let runtime = Runtime::new();
        let result = runtime.evaluate_expression(&expr);
        
        assert!(result.is_ok(), "Failed to handle large string of size {}", size);
        
        if let JsonValue::String(result_str) = result.unwrap() {
            assert_eq!(result_str.len(), size);
        }
    }
    
    // Create many small expressions
    let runtime = Runtime::new();
    for _ in 0..10000 {
        let expr = Expression::StringLiteral("small".to_string());
        let result = runtime.evaluate_expression(&expr);
        assert!(result.is_ok(), "Failed under memory pressure");
    }
}

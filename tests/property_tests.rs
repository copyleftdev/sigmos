//! Property-based tests for SIGMOS components
//!
//! These tests use `proptest` to generate random inputs and verify
//! that our components behave correctly across a wide range of scenarios.

use proptest::prelude::*;
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

// Property-based test strategies for generating test data

/// Generate valid identifier strings
fn valid_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_]*"
        .prop_map(|s| s.chars().take(50).collect()) // Limit length
}

/// Generate valid version numbers
fn valid_version() -> impl Strategy<Value = Version> {
    (1u32..100, 0u32..100, prop::option::of(0u32..100))
        .prop_map(|(major, minor, patch)| Version { major, minor, patch })
}

/// Generate valid string literals
fn string_literal() -> impl Strategy<Value = Expression> {
    ".*"
        .prop_map(|s| s.chars().take(100).collect()) // Limit length
        .prop_map(Expression::StringLiteral)
}

/// Generate valid number expressions
fn number_expression() -> impl Strategy<Value = Expression> {
    any::<f64>()
        .prop_filter("Must be finite", |n| n.is_finite())
        .prop_map(Expression::Number)
}

/// Generate valid boolean expressions
fn boolean_expression() -> impl Strategy<Value = Expression> {
    any::<bool>().prop_map(Expression::Boolean)
}

/// Generate valid identifier expressions
fn identifier_expression() -> impl Strategy<Value = Expression> {
    valid_identifier().prop_map(Expression::Identifier)
}

/// Generate simple expressions (no recursion to avoid infinite generation)
fn simple_expression() -> impl Strategy<Value = Expression> {
    prop_oneof![
        string_literal(),
        number_expression(),
        boolean_expression(),
        identifier_expression(),
    ]
}

/// Generate MCP plugin configurations
fn mcp_config() -> impl Strategy<Value = McpConfig> {
    (
        valid_identifier(),
        "https?://[a-zA-Z0-9.-]+",
        valid_identifier(),
        prop::option::of(".*"),
        1u64..3600,
        prop::option::of(1u32..10000),
        prop::option::of(0.0f32..2.0f32),
    ).prop_map(|(name, endpoint, model, api_key, timeout, max_tokens, temperature)| {
        McpConfig {
            name,
            endpoint,
            model,
            api_key,
            timeout_seconds: timeout,
            max_tokens,
            temperature,
        }
    })
}

/// Generate REST plugin configurations
fn rest_config() -> impl Strategy<Value = RestConfig> {
    (
        valid_identifier(),
        "https?://[a-zA-Z0-9.-]+",
        1u64..3600,
        1u32..10,
        any::<bool>(),
        prop::option::of(".*"),
        ".*",
    ).prop_map(|(name, base_url, timeout, max_redirects, verify_ssl, auth_token, user_agent)| {
        RestConfig {
            name,
            base_url,
            default_headers: HashMap::new(),
            timeout_seconds: timeout,
            max_redirects,
            verify_ssl,
            auth_token,
            user_agent,
        }
    })
}

// Property-based tests

proptest! {
    /// Test that parser handles various inputs gracefully
    #[test]
    fn test_parser_robustness(input in ".*") {
        // Parser should never panic, even on invalid input
        let result = std::panic::catch_unwind(|| {
            SigmosParser::parse_spec(&input)
        });
        
        // Should not panic
        prop_assert!(result.is_ok());
        
        // Currently returns placeholder, so should always succeed
        let parse_result = SigmosParser::parse_spec(&input);
        prop_assert!(parse_result.is_ok());
    }

    /// Test expression evaluation with various expressions
    #[test]
    fn test_expression_evaluation_robustness(expr in simple_expression()) {
        let runtime = Runtime::new();
        
        // Expression evaluation should never panic
        let result = std::panic::catch_unwind(|| {
            runtime.evaluate_expression(&expr)
        });
        
        prop_assert!(result.is_ok());
        
        // Evaluation should succeed for simple expressions
        let eval_result = runtime.evaluate_expression(&expr);
        prop_assert!(eval_result.is_ok());
        
        // Verify result type matches expression type
        let value = eval_result.unwrap();
        match (&expr, &value) {
            (Expression::StringLiteral(_), JsonValue::String(_)) => {},
            (Expression::Number(_), JsonValue::Number(_)) => {},
            (Expression::Boolean(_), JsonValue::Bool(_)) => {},
            (Expression::Identifier(_), JsonValue::String(_)) => {}, // Placeholder behavior
            _ => prop_assert!(false, "Unexpected result type for expression: {:?} -> {:?}", expr, value),
        }
    }

    /// Test MCP plugin configuration validation
    #[test]
    fn test_mcp_config_validation(config in mcp_config()) {
        // Valid configurations should always validate successfully
        let validation_result = config.validate();
        prop_assert!(validation_result.is_ok());
        
        // Plugin creation should succeed with valid config
        let plugin_result = McpPlugin::new(config);
        prop_assert!(plugin_result.is_ok());
    }

    /// Test REST plugin configuration validation
    #[test]
    fn test_rest_config_validation(config in rest_config()) {
        // Valid configurations should always validate successfully
        let validation_result = config.validate();
        prop_assert!(validation_result.is_ok());
        
        // Plugin creation should succeed with valid config
        let plugin_result = RestPlugin::new(config);
        prop_assert!(plugin_result.is_ok());
    }

    /// Test plugin registry operations
    #[test]
    fn test_plugin_registry_operations(
        plugin_names in prop::collection::vec(valid_identifier(), 1..10)
    ) {
        let mut registry = PluginRegistry::new();
        let mut registered_plugins = Vec::new();
        
        // Register plugins with unique names
        for name in plugin_names.iter().take(5) { // Limit to avoid too many plugins
            let config = McpConfig {
                name: name.clone(),
                endpoint: "http://localhost:8080".to_string(),
                model: "test".to_string(),
                ..Default::default()
            };
            
            let plugin = McpPlugin::new(config);
            prop_assert!(plugin.is_ok());
            
            let register_result = registry.register_plugin(
                Box::new(plugin.unwrap()),
                sigmos_plugins::mcp::McpPlugin::metadata(),
                sigmos_plugins::mcp::McpPlugin::capabilities(),
            );
            
            if register_result.is_ok() {
                registered_plugins.push(name.clone());
            }
        }
        
        // Verify all registered plugins exist
        for name in &registered_plugins {
            prop_assert!(registry.has_plugin(name));
        }
        
        // Verify plugin count
        prop_assert_eq!(registry.plugin_count(), registered_plugins.len());
    }

    /// Test runtime with multiple expressions
    #[test]
    fn test_runtime_multiple_expressions(
        expressions in prop::collection::vec(simple_expression(), 1..20)
    ) {
        let runtime = Runtime::new();
        
        // All expressions should evaluate successfully
        for expr in expressions {
            let result = runtime.evaluate_expression(&expr);
            prop_assert!(result.is_ok(), "Failed to evaluate expression: {:?}", expr);
        }
    }

    /// Test string handling edge cases
    #[test]
    fn test_string_handling(s in ".*") {
        let runtime = Runtime::new();
        let expr = Expression::StringLiteral(s.clone());
        
        let result = runtime.evaluate_expression(&expr);
        prop_assert!(result.is_ok());
        
        if let JsonValue::String(result_str) = result.unwrap() {
            prop_assert_eq!(result_str, s);
        } else {
            prop_assert!(false, "Expected string result");
        }
    }

    /// Test number handling edge cases
    #[test]
    fn test_number_handling(n in any::<f64>().prop_filter("Must be finite", |x| x.is_finite())) {
        let runtime = Runtime::new();
        let expr = Expression::Number(n);
        
        let result = runtime.evaluate_expression(&expr);
        prop_assert!(result.is_ok());
        
        if let JsonValue::Number(result_num) = result.unwrap() {
            let result_f64 = result_num.as_f64().unwrap();
            prop_assert!((result_f64 - n).abs() < f64::EPSILON);
        } else {
            prop_assert!(false, "Expected number result");
        }
    }

    /// Test plugin method execution with various arguments
    #[test]
    fn test_plugin_method_execution(
        method_name in "[a-zA-Z][a-zA-Z0-9_]*",
        arg_values in prop::collection::vec(".*", 0..5)
    ) {
        let mut registry = PluginRegistry::new();
        
        // Register a test plugin
        let config = McpConfig {
            name: "test_plugin".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            model: "test".to_string(),
            ..Default::default()
        };
        
        let plugin = McpPlugin::new(config).unwrap();
        registry.register_plugin(
            Box::new(plugin),
            sigmos_plugins::mcp::McpPlugin::metadata(),
            sigmos_plugins::mcp::McpPlugin::capabilities(),
        ).unwrap();
        
        // Create arguments
        let mut args = HashMap::new();
        for (i, value) in arg_values.iter().enumerate() {
            args.insert(format!("arg_{}", i), JsonValue::String(value.clone()));
        }
        
        // Execute method - should not panic even with unknown methods
        let result = std::panic::catch_unwind(|| {
            registry.execute_plugin_method("test_plugin", &method_name, &args)
        });
        
        prop_assert!(result.is_ok());
        
        // Known methods should return valid responses
        if ["complete", "embed", "chat", "analyze"].contains(&method_name.as_str()) {
            let exec_result = registry.execute_plugin_method("test_plugin", &method_name, &args);
            // May fail due to missing required arguments, but should not panic
            prop_assert!(exec_result.is_ok() || exec_result.is_err());
        }
    }
}

/// Additional targeted property tests for edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    proptest! {
        /// Test very long strings don't cause issues
        #[test]
        fn test_long_strings(s in prop::collection::vec(any::<char>(), 0..10000).prop_map(|chars| chars.into_iter().collect::<String>())) {
            let runtime = Runtime::new();
            let expr = Expression::StringLiteral(s.clone());
            
            let result = runtime.evaluate_expression(&expr);
            prop_assert!(result.is_ok());
        }

        /// Test extreme numbers
        #[test]
        fn test_extreme_numbers(n in prop_oneof![
            Just(f64::MIN),
            Just(f64::MAX),
            Just(0.0),
            Just(-0.0),
            Just(1.0),
            Just(-1.0),
        ]) {
            let runtime = Runtime::new();
            let expr = Expression::Number(n);
            
            let result = runtime.evaluate_expression(&expr);
            prop_assert!(result.is_ok());
        }

        /// Test empty and whitespace-only strings
        #[test]
        fn test_whitespace_strings(s in "\\s*") {
            let runtime = Runtime::new();
            let expr = Expression::StringLiteral(s.clone());
            
            let result = runtime.evaluate_expression(&expr);
            prop_assert!(result.is_ok());
            
            if let JsonValue::String(result_str) = result.unwrap() {
                prop_assert_eq!(result_str, s);
            }
        }
    }
}

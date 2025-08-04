//! Benchmark tests for SIGMOS components
//!
//! These tests measure performance characteristics of our core components
//! to ensure they meet performance requirements and detect regressions.

use sigmos_core::{SigmosParser, ast::*};
use sigmos_runtime::Runtime;
use sigmos_plugins::{
    registry::PluginRegistry,
    mcp::{McpPlugin, McpConfig},
    rest::{RestPlugin, RestConfig},
    ConfigurablePlugin,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde_json::Value as JsonValue;

/// Benchmark result structure
#[derive(Debug, Clone)]
struct BenchmarkResult {
    name: String,
    iterations: usize,
    total_time: Duration,
    avg_time: Duration,
    min_time: Duration,
    max_time: Duration,
    ops_per_second: f64,
}

impl BenchmarkResult {
    fn new(name: String, iterations: usize, times: Vec<Duration>) -> Self {
        let total_time: Duration = times.iter().sum();
        let avg_time = total_time / iterations as u32;
        let min_time = *times.iter().min().unwrap();
        let max_time = *times.iter().max().unwrap();
        let ops_per_second = iterations as f64 / total_time.as_secs_f64();
        
        Self {
            name,
            iterations,
            total_time,
            avg_time,
            min_time,
            max_time,
            ops_per_second,
        }
    }
    
    fn print(&self) {
        println!("Benchmark: {}", self.name);
        println!("  Iterations: {}", self.iterations);
        println!("  Total time: {:?}", self.total_time);
        println!("  Average time: {:?}", self.avg_time);
        println!("  Min time: {:?}", self.min_time);
        println!("  Max time: {:?}", self.max_time);
        println!("  Ops/sec: {:.2}", self.ops_per_second);
        println!();
    }
}

/// Run a benchmark with the given closure
fn benchmark<F>(name: &str, iterations: usize, mut f: F) -> BenchmarkResult
where
    F: FnMut() -> (),
{
    let mut times = Vec::with_capacity(iterations);
    
    // Warm up
    for _ in 0..10 {
        f();
    }
    
    // Actual benchmark
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        let elapsed = start.elapsed();
        times.push(elapsed);
    }
    
    BenchmarkResult::new(name.to_string(), iterations, times)
}

/// Benchmark parser performance
#[test]
fn benchmark_parser_performance() {
    let test_specs = vec![
        r#"spec "Simple" v1.0 {}"#,
        r#"spec "WithDescription" v1.2.3 { description: "Test spec" }"#,
        r#"spec "Complex" v2.1.0 { 
            description: "A more complex specification"
            author: "SIGMOS Team"
            version: "2.1.0"
        }"#,
    ];
    
    for (i, spec) in test_specs.iter().enumerate() {
        let result = benchmark(
            &format!("Parser - Spec {}", i + 1),
            1000,
            || {
                let _ = SigmosParser::parse_spec(spec);
            }
        );
        result.print();
        
        // Assert performance requirements
        assert!(result.avg_time < Duration::from_millis(1), 
                "Parser too slow: {:?} > 1ms", result.avg_time);
        assert!(result.ops_per_second > 1000.0, 
                "Parser throughput too low: {:.2} < 1000 ops/sec", result.ops_per_second);
    }
}

/// Benchmark expression evaluation performance
#[test]
fn benchmark_expression_evaluation() {
    let runtime = Runtime::new();
    
    let test_expressions = vec![
        ("String Literal", Expression::StringLiteral("Hello, World!".to_string())),
        ("Number", Expression::Number(42.0)),
        ("Boolean", Expression::Boolean(true)),
        ("Identifier", Expression::Identifier("test_var".to_string())),
        ("Long String", Expression::StringLiteral("x".repeat(1000))),
        ("Large Number", Expression::Number(f64::MAX / 2.0)),
    ];
    
    for (name, expr) in test_expressions {
        let result = benchmark(
            &format!("Expression Eval - {}", name),
            10000,
            || {
                let _ = runtime.evaluate_expression(&expr);
            }
        );
        result.print();
        
        // Assert performance requirements
        assert!(result.avg_time < Duration::from_micros(100), 
                "Expression evaluation too slow: {:?} > 100μs", result.avg_time);
        assert!(result.ops_per_second > 10000.0, 
                "Expression evaluation throughput too low: {:.2} < 10000 ops/sec", result.ops_per_second);
    }
}

/// Benchmark plugin creation and initialization
#[test]
fn benchmark_plugin_creation() {
    // Benchmark MCP plugin creation
    let mcp_config = McpConfig {
        name: "benchmark_mcp".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        model: "test".to_string(),
        ..Default::default()
    };
    
    let result = benchmark(
        "MCP Plugin Creation",
        1000,
        || {
            let plugin = McpPlugin::new(mcp_config.clone()).unwrap();
            std::mem::drop(plugin);
        }
    );
    result.print();
    
    assert!(result.avg_time < Duration::from_millis(1), 
            "MCP plugin creation too slow: {:?} > 1ms", result.avg_time);
    
    // Benchmark REST plugin creation
    let rest_config = RestConfig {
        name: "benchmark_rest".to_string(),
        base_url: "https://api.example.com".to_string(),
        ..Default::default()
    };
    
    let result = benchmark(
        "REST Plugin Creation",
        1000,
        || {
            let plugin = RestPlugin::new(rest_config.clone()).unwrap();
            std::mem::drop(plugin);
        }
    );
    result.print();
    
    assert!(result.avg_time < Duration::from_millis(1), 
            "REST plugin creation too slow: {:?} > 1ms", result.avg_time);
}

/// Benchmark plugin registry operations
#[test]
fn benchmark_plugin_registry() {
    // Setup registry with plugins
    let mut registry = PluginRegistry::new();
    
    for i in 0..10 {
        let config = McpConfig {
            name: format!("plugin_{}", i),
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
    }
    
    // Benchmark plugin lookup
    let result = benchmark(
        "Plugin Registry Lookup",
        10000,
        || {
            let _ = registry.has_plugin("plugin_5");
        }
    );
    result.print();
    
    assert!(result.avg_time < Duration::from_micros(10), 
            "Plugin lookup too slow: {:?} > 10μs", result.avg_time);
    
    // Benchmark plugin method execution
    let mut args = HashMap::new();
    args.insert("prompt".to_string(), JsonValue::String("test".to_string()));
    
    let result = benchmark(
        "Plugin Method Execution",
        1000,
        || {
            let _ = registry.execute_plugin_method("plugin_0", "complete", &args);
        }
    );
    result.print();
    
    assert!(result.avg_time < Duration::from_millis(1), 
            "Plugin method execution too slow: {:?} > 1ms", result.avg_time);
}

/// Benchmark memory allocation patterns
#[test]
fn benchmark_memory_allocation() {
    // Benchmark runtime creation
    let result = benchmark(
        "Runtime Creation",
        1000,
        || {
            let runtime = Runtime::new();
            std::mem::drop(runtime);
        }
    );
    result.print();
    
    assert!(result.avg_time < Duration::from_micros(100), 
            "Runtime creation too slow: {:?} > 100μs", result.avg_time);
    
    // Benchmark plugin registry creation
    let result = benchmark(
        "Plugin Registry Creation",
        1000,
        || {
            let registry = PluginRegistry::new();
            std::mem::drop(registry);
        }
    );
    result.print();
    
    assert!(result.avg_time < Duration::from_micros(50), 
            "Plugin registry creation too slow: {:?} > 50μs", result.avg_time);
}

/// Benchmark concurrent operations
#[test]
fn benchmark_concurrent_operations() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let registry = Arc::new(Mutex::new(PluginRegistry::new()));
    
    // Register a plugin
    {
        let mut reg = registry.lock().unwrap();
        let config = McpConfig {
            name: "concurrent_plugin".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            model: "test".to_string(),
            ..Default::default()
        };
        
        let plugin = McpPlugin::new(config).unwrap();
        reg.register_plugin(
            Box::new(plugin),
            sigmos_plugins::mcp::McpPlugin::metadata(),
            sigmos_plugins::mcp::McpPlugin::capabilities(),
        ).unwrap();
    }
    
    // Benchmark concurrent plugin execution
    let start = Instant::now();
    let handles: Vec<_> = (0..10).map(|i| {
        let registry_clone = Arc::clone(&registry);
        thread::spawn(move || {
            for _ in 0..100 {
                let reg = registry_clone.lock().unwrap();
                let mut args = HashMap::new();
                args.insert("prompt".to_string(), JsonValue::String(format!("test_{}", i)));
                let _ = reg.execute_plugin_method("concurrent_plugin", "complete", &args);
            }
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let total_time = start.elapsed();
    let total_ops = 10 * 100;
    let ops_per_second = total_ops as f64 / total_time.as_secs_f64();
    
    println!("Concurrent Operations Benchmark:");
    println!("  Total operations: {}", total_ops);
    println!("  Total time: {:?}", total_time);
    println!("  Ops/sec: {:.2}", ops_per_second);
    println!();
    
    assert!(ops_per_second > 1000.0, 
            "Concurrent operations too slow: {:.2} < 1000 ops/sec", ops_per_second);
}

/// Benchmark scaling characteristics
#[test]
fn benchmark_scaling() {
    // Test parser scaling with input size
    for size in [100, 1000, 10000] {
        let large_spec = format!(
            r#"spec "Large" v1.0 {{ description: "{}" }}"#,
            "x".repeat(size)
        );
        
        let result = benchmark(
            &format!("Parser - Input Size {}", size),
            100,
            || {
                let _ = SigmosParser::parse_spec(&large_spec);
            }
        );
        result.print();
        
        // Performance should scale reasonably with input size
        let expected_max_time = Duration::from_micros(size as u64 / 10);
        assert!(result.avg_time < expected_max_time, 
                "Parser scaling poor for size {}: {:?} > {:?}", 
                size, result.avg_time, expected_max_time);
    }
    
    // Test plugin registry scaling with number of plugins
    for plugin_count in [10, 50, 100] {
        let mut registry = PluginRegistry::new();
        
        // Register plugins
        for i in 0..plugin_count {
            let config = McpConfig {
                name: format!("plugin_{}", i),
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
        }
        
        // Benchmark lookup performance
        let result = benchmark(
            &format!("Registry Lookup - {} Plugins", plugin_count),
            1000,
            || {
                let _ = registry.has_plugin(&format!("plugin_{}", plugin_count / 2));
            }
        );
        result.print();
        
        // Lookup should be O(1) or close to it
        assert!(result.avg_time < Duration::from_micros(50), 
                "Registry lookup scaling poor for {} plugins: {:?} > 50μs", 
                plugin_count, result.avg_time);
    }
}

/// Performance regression test
#[test]
fn test_performance_regression() {
    // This test establishes baseline performance expectations
    // and will catch significant performance regressions
    
    let runtime = Runtime::new();
    let expr = Expression::StringLiteral("performance test".to_string());
    
    // Measure baseline performance
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = runtime.evaluate_expression(&expr);
    }
    let elapsed = start.elapsed();
    
    let ops_per_second = 10000.0 / elapsed.as_secs_f64();
    
    println!("Performance Regression Test:");
    println!("  Expression evaluations per second: {:.2}", ops_per_second);
    
    // Establish minimum performance threshold
    assert!(ops_per_second > 50000.0, 
            "Performance regression detected: {:.2} < 50000 ops/sec", ops_per_second);
    
    // Test parser performance baseline
    let spec = r#"spec "PerfTest" v1.0 { description: "Performance regression test" }"#;
    
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = SigmosParser::parse_spec(spec);
    }
    let elapsed = start.elapsed();
    
    let parse_ops_per_second = 1000.0 / elapsed.as_secs_f64();
    
    println!("  Parser operations per second: {:.2}", parse_ops_per_second);
    
    assert!(parse_ops_per_second > 5000.0, 
            "Parser performance regression detected: {:.2} < 5000 ops/sec", parse_ops_per_second);
}

# SIGMOS Developer Guide

This guide is for developers who want to contribute to SIGMOS, extend its functionality, or understand its internal architecture.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Project Structure](#project-structure)
3. [Development Setup](#development-setup)
4. [Core Components](#core-components)
5. [Plugin Development](#plugin-development)
6. [Testing](#testing)
7. [Contributing](#contributing)
8. [API Reference](#api-reference)

## Architecture Overview

SIGMOS follows a modular architecture with clear separation of concerns:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│      CLI        │    │   Transpiler    │    │     Plugins     │
│   (User I/O)    │    │ (JSON/YAML/etc) │    │ (MCP, REST, etc)│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │     Runtime     │
                    │  (Execution)    │
                    └─────────────────┘
                                 │
                    ┌─────────────────┐
                    │      Core       │
                    │ (Parser, AST)   │
                    └─────────────────┘
```

### Key Principles

- **Zero unsafe code**: All code must be memory-safe Rust
- **Modular design**: Each crate has a single responsibility
- **Extensive testing**: Unit, integration, property-based, and fuzz tests
- **Strong typing**: Leverage Rust's type system for correctness
- **Documentation**: All public APIs must be documented with examples

## Project Structure

```
sigmos/
├── crates/
│   ├── core/           # Parser, AST, type system
│   ├── runtime/        # Expression evaluation, execution engine
│   ├── cli/            # Command-line interface
│   ├── plugins/        # Official plugins (MCP, REST)
│   └── transpiler/     # Output format conversion
├── docs/               # Documentation
├── examples/           # Example SIGMOS specifications
├── tests/              # Integration and property-based tests
└── spec/               # Language specification
```

### Crate Dependencies

```
cli ──┐
      ├─→ runtime ──→ core
      └─→ transpiler ──→ core
      
plugins ──→ runtime ──→ core
```

## Development Setup

### Prerequisites

- Rust 1.70+ (latest stable recommended)
- Git
- A good text editor or IDE (VS Code with rust-analyzer recommended)

### Setup Steps

```bash
# Clone the repository
git clone https://github.com/copyleftdev/sigmos
cd sigmos

# Install dependencies and build
cargo build

# Run tests
cargo test

# Run linting
cargo clippy -- -D warnings

# Format code
cargo fmt

# Generate documentation
cargo doc --open
```

### Development Workflow

1. **Create a feature branch**: `git checkout -b feature/my-feature`
2. **Write tests first**: Follow TDD principles
3. **Implement the feature**: Write clean, documented code
4. **Run the full test suite**: `cargo test`
5. **Check linting**: `cargo clippy`
6. **Format code**: `cargo fmt`
7. **Update documentation**: Add/update docs as needed
8. **Create a pull request**: Include tests and documentation

## Core Components

### 1. Parser (crates/core)

The parser converts SIGMOS source code into an Abstract Syntax Tree (AST).

#### Key Files
- `grammar.pest`: PEG grammar definition
- `parser.rs`: Parser implementation using Pest
- `ast.rs`: AST node definitions

#### Adding New Syntax

1. **Update the grammar** in `grammar.pest`:
```pest
// Add new rule
new_feature = { "keyword" ~ identifier ~ "{" ~ content ~ "}" }
```

2. **Update the AST** in `ast.rs`:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewFeature {
    pub name: String,
    pub content: String,
}
```

3. **Update the parser** in `parser.rs`:
```rust
fn parse_new_feature(pair: Pair<Rule>) -> Result<NewFeature, ParseError> {
    // Implementation
}
```

4. **Add tests**:
```rust
#[test]
fn test_parse_new_feature() {
    let input = r#"keyword example { content }"#;
    let result = SigmosParser::parse_new_feature(input);
    assert!(result.is_ok());
}
```

### 2. Runtime (crates/runtime)

The runtime executes SIGMOS specifications and evaluates expressions.

#### Key Components

- **Expression Evaluation**: Handles all expression types
- **Plugin System**: Manages plugin registration and execution
- **Context Management**: Variable scoping and lifecycle
- **Error Handling**: Comprehensive error reporting

#### Expression Evaluation Architecture

```rust
pub enum Expression {
    // Literals
    StringLiteral(String),
    Number(f64),
    Boolean(bool),
    
    // Variables and functions
    Identifier(String),
    FunctionCall { object: String, method: String, arguments: Vec<Argument> },
    
    // Operators
    Add(Box<Expression>, Box<Expression>),
    // ... other operators
    
    // Advanced features
    Conditional { condition: Box<Expression>, if_true: Box<Expression>, if_false: Box<Expression> },
    StringTemplate { parts: Vec<TemplatePart> },
}
```

#### Adding New Expression Types

1. **Extend the Expression enum** in `core/src/ast.rs`
2. **Update the evaluation logic** in `runtime/src/lib.rs`
3. **Add comprehensive tests**

Example:
```rust
// In ast.rs
pub enum Expression {
    // ... existing variants
    NewOperation(Box<Expression>, String), // New expression type
}

// In runtime/src/lib.rs
impl Runtime {
    fn evaluate_expression_with_context(&self, expr: &Expression, context: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue> {
        match expr {
            // ... existing cases
            Expression::NewOperation(operand, operation) => {
                let value = self.evaluate_expression_with_context(operand, context)?;
                self.perform_new_operation(&value, operation)
            }
        }
    }
    
    fn perform_new_operation(&self, value: &JsonValue, operation: &str) -> RuntimeResult<JsonValue> {
        // Implementation
    }
}
```

### 3. Plugin System (crates/plugins)

The plugin system allows extending SIGMOS with custom functionality.

#### Plugin Architecture

```rust
pub trait Plugin: std::fmt::Debug {
    fn name(&self) -> &str;
    fn initialize(&mut self) -> RuntimeResult<()>;
    fn execute(&self, method: &str, args: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue>;
}

pub trait ConfigurablePlugin<C: PluginConfig>: Plugin {
    fn new(config: C) -> Result<Self, PluginError> where Self: Sized;
    fn config(&self) -> &C;
}
```

#### Creating a New Plugin

1. **Define the plugin struct**:
```rust
#[derive(Debug)]
pub struct MyPlugin {
    config: MyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyConfig {
    pub name: String,
    pub endpoint: String,
    // ... other config fields
}
```

2. **Implement the configuration trait**:
```rust
impl PluginConfig for MyConfig {
    fn validate(&self) -> Result<(), PluginError> {
        if self.name.is_empty() {
            return Err(PluginError::InvalidConfig("Name cannot be empty".to_string()));
        }
        // ... other validations
        Ok(())
    }
    
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            description: "My custom plugin".to_string(),
            author: "Your Name".to_string(),
        }
    }
    
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![
            PluginCapability {
                name: "my_method".to_string(),
                description: "Does something useful".to_string(),
                parameters: vec![
                    ParameterSpec {
                        name: "input".to_string(),
                        param_type: "String".to_string(),
                        required: true,
                        description: "Input parameter".to_string(),
                    }
                ],
            }
        ]
    }
}
```

3. **Implement the plugin traits**:
```rust
impl ConfigurablePlugin<MyConfig> for MyPlugin {
    fn new(config: MyConfig) -> Result<Self, PluginError> {
        config.validate()?;
        Ok(Self { config })
    }
    
    fn config(&self) -> &MyConfig {
        &self.config
    }
}

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn initialize(&mut self) -> RuntimeResult<()> {
        // Initialization logic
        Ok(())
    }
    
    fn execute(&self, method: &str, args: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue> {
        match method {
            "my_method" => {
                let input = args.get("input")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| RuntimeError::Evaluation("Missing input parameter".to_string()))?;
                
                // Your plugin logic here
                Ok(JsonValue::String(format!("Processed: {}", input)))
            }
            _ => Err(RuntimeError::Evaluation(format!("Unknown method: {}", method)))
        }
    }
}
```

4. **Add tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_plugin_creation() {
        let config = MyConfig {
            name: "test_plugin".to_string(),
            endpoint: "http://localhost:8080".to_string(),
        };
        
        let plugin = MyPlugin::new(config);
        assert!(plugin.is_ok());
    }
    
    #[test]
    fn test_my_plugin_execution() {
        let config = MyConfig {
            name: "test_plugin".to_string(),
            endpoint: "http://localhost:8080".to_string(),
        };
        
        let plugin = MyPlugin::new(config).unwrap();
        let mut args = HashMap::new();
        args.insert("input".to_string(), JsonValue::String("test".to_string()));
        
        let result = plugin.execute("my_method", &args);
        assert!(result.is_ok());
    }
}
```

## Testing

SIGMOS has a comprehensive testing strategy:

### Test Types

1. **Unit Tests**: Test individual functions and methods
2. **Integration Tests**: Test component interactions
3. **Property-Based Tests**: Test with generated inputs using `proptest`
4. **Fuzz Tests**: Test with random/malformed inputs
5. **Benchmark Tests**: Performance testing and regression detection

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test --package sigmos-core

# Run integration tests
cargo test --tests

# Run property-based tests
cargo test property_tests

# Run benchmarks
cargo test benchmarks -- --nocapture
```

### Writing Tests

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test input";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
}
```

#### Property-Based Tests
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property(input in ".*") {
        // Test that some property holds for all inputs
        let result = function_under_test(&input);
        prop_assert!(result.is_ok());
    }
}
```

#### Integration Tests
```rust
// In tests/integration_tests.rs
#[test]
fn test_full_workflow() {
    let spec = r#"
    spec "Test" v1.0.0 {
        input x: Number = 5
        computed y: Number = x * 2
    }
    "#;
    
    let parsed = SigmosParser::parse_spec(spec).unwrap();
    let runtime = Runtime::new();
    let result = runtime.execute(&parsed).await.unwrap();
    
    // Assert expected behavior
}
```

## Contributing

### Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Write comprehensive documentation
- Include examples in doc comments

### Documentation Standards

```rust
/// Brief description of the function
///
/// Longer description explaining the purpose, behavior, and any important
/// details about the function.
///
/// # Arguments
///
/// * `param1` - Description of the first parameter
/// * `param2` - Description of the second parameter
///
/// # Returns
///
/// Description of what the function returns
///
/// # Errors
///
/// Description of when and why the function might return an error
///
/// # Examples
///
/// ```rust
/// use sigmos_core::SigmosParser;
///
/// let result = SigmosParser::parse_spec("spec \"Test\" v1.0.0 {}");
/// assert!(result.is_ok());
/// ```
pub fn example_function(param1: &str, param2: i32) -> Result<String, Error> {
    // Implementation
}
```

### Pull Request Process

1. **Fork the repository** and create a feature branch
2. **Write tests** for your changes
3. **Implement your feature** following the coding standards
4. **Update documentation** as needed
5. **Run the full test suite** and ensure all tests pass
6. **Create a pull request** with:
   - Clear description of the changes
   - Reference to any related issues
   - Screenshots or examples if applicable

### Commit Message Format

```
type(scope): brief description

Longer description explaining the change in detail.

- List any breaking changes
- Reference related issues (#123)
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## API Reference

### Core Types

#### Spec
```rust
pub struct Spec {
    pub name: String,
    pub version: Version,
    pub description: Option<String>,
    pub inputs: Vec<Field>,
    pub computed: Vec<ComputedField>,
    pub events: Vec<EventDef>,
    pub constraints: Vec<ConstraintDef>,
    pub lifecycle: Vec<LifecycleDef>,
    pub extensions: Vec<Extension>,
    pub types: Vec<TypeDef>,
}
```

#### Expression
```rust
pub enum Expression {
    StringLiteral(String),
    Number(f64),
    Boolean(bool),
    Identifier(String),
    FunctionCall { object: String, method: String, arguments: Vec<Argument> },
    // Arithmetic operators
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    // ... other variants
}
```

### Runtime API

#### Runtime
```rust
impl Runtime {
    pub fn new() -> Self;
    pub fn register_plugin(&mut self, name: String, plugin: Box<dyn Plugin>);
    pub fn evaluate_expression(&self, expr: &Expression) -> RuntimeResult<JsonValue>;
    pub fn evaluate_expression_with_context(&self, expr: &Expression, context: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue>;
    pub async fn execute(&mut self, spec: &Spec) -> RuntimeResult<()>;
}
```

### Plugin API

#### Plugin Trait
```rust
pub trait Plugin: std::fmt::Debug {
    fn name(&self) -> &str;
    fn initialize(&mut self) -> RuntimeResult<()>;
    fn execute(&self, method: &str, args: &HashMap<String, JsonValue>) -> RuntimeResult<JsonValue>;
}
```

#### ConfigurablePlugin Trait
```rust
pub trait ConfigurablePlugin<C: PluginConfig>: Plugin {
    fn new(config: C) -> Result<Self, PluginError> where Self: Sized;
    fn config(&self) -> &C;
}
```

## Performance Considerations

### Expression Evaluation
- Expression evaluation is designed to be fast with minimal allocations
- Complex expressions are evaluated recursively but with stack safety
- Built-in functions are optimized for common use cases

### Plugin System
- Plugins are loaded once and reused across evaluations
- Plugin method calls have minimal overhead
- Thread-safe plugin access using `Arc<RwLock<>>`

### Memory Management
- All data structures use owned types to avoid lifetime issues
- JSON values are used for runtime data to provide flexibility
- Careful use of `Box<>` and `Arc<>` to minimize memory usage

## Debugging

### Logging
```rust
use log::{debug, info, warn, error};

debug!("Parsing expression: {:?}", expr);
info!("Plugin registered: {}", plugin_name);
warn!("Variable not found: {}", var_name);
error!("Evaluation failed: {}", error);
```

### Error Handling
```rust
// Use specific error types
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
}

// Provide context in error messages
Err(MyError::Runtime(format!("Failed to evaluate expression: {:?}", expr)))
```

### Testing and Debugging Tips

1. **Use `cargo test -- --nocapture`** to see print statements in tests
2. **Use `RUST_LOG=debug cargo test`** for detailed logging
3. **Write focused unit tests** for specific functionality
4. **Use property-based tests** to find edge cases
5. **Add debug prints** in complex evaluation logic

---

This developer guide should help you understand SIGMOS internals and contribute effectively to the project. For more specific questions, check the API documentation or reach out to the maintainers.

*Happy hacking! 🦀*

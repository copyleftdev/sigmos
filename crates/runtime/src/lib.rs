//! # SIGMOS Runtime
//!
//! Evaluation engine and orchestration for SIGMOS specifications.
//!
//! This crate provides the runtime environment for executing SIGMOS specs:
//! - Specification execution engine
//! - Event handling and lifecycle management
//! - Plugin system integration
//! - Async orchestration
//!
//! # Examples
//!
//! ```rust
//! use sigmos_runtime::Runtime;
//! use sigmos_core::ast::Spec;
//!
//! # tokio_test::block_on(async {
//! let mut runtime = Runtime::new();
//! // let spec = parse_spec(input)?;
//! // runtime.execute(&spec).await?;
//! # });
//! ```

use serde_json::Value as JsonValue;
use sigmos_core::ast::*;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

pub mod engine;
pub mod events;
pub mod lifecycle;
pub mod plugins;

/// Runtime errors
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Execution failed: {0}")]
    Execution(String),
    #[error("Plugin error: {0}")]
    Plugin(String),
    #[error("Expression evaluation error: {0}")]
    Evaluation(String),
    #[error("Event handling error: {0}")]
    Event(String),
    #[error("Lifecycle error: {0}")]
    Lifecycle(String),
}

/// Result type for runtime operations
pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// SIGMOS runtime execution engine
///
/// The runtime manages the execution of SIGMOS specifications,
/// handling events, lifecycle phases, and plugin orchestration.
///
/// # Examples
///
/// ```rust
/// use sigmos_runtime::Runtime;
///
/// # tokio_test::block_on(async {
/// let mut runtime = Runtime::new();
/// // Execute a specification
/// # });
/// ```

// Helper enums for expression evaluation operations
#[derive(Debug, Clone, Copy)]
enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug, Clone, Copy)]
enum ComparisonOp {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

pub struct Runtime {
    /// Execution context
    context: Arc<RwLock<ExecutionContext>>,
    /// Registered plugins
    plugins: HashMap<String, Box<dyn Plugin + Send + Sync>>,
    /// Event handlers
    #[allow(dead_code)]
    event_handlers: HashMap<String, Vec<EventHandler>>,
}

/// Execution context for runtime
#[derive(Debug, Default)]
pub struct ExecutionContext {
    /// Variable bindings
    variables: HashMap<String, serde_json::Value>,
    /// Computed values cache
    computed_cache: HashMap<String, serde_json::Value>,
    /// Execution state
    state: ExecutionState,
}

/// Execution state
#[derive(Debug, Default)]
pub enum ExecutionState {
    #[default]
    Idle,
    Running,
    Completed,
    Failed(String),
}

/// Plugin trait for extending runtime functionality
///
/// Note: Async methods are not object-safe, so we use a simpler synchronous interface
pub trait Plugin: std::fmt::Debug {
    /// Plugin name
    fn name(&self) -> &str;

    /// Initialize the plugin
    fn initialize(&mut self) -> RuntimeResult<()>;

    /// Execute a plugin method
    fn execute(
        &self,
        method: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> RuntimeResult<serde_json::Value>;
}

/// Event handler function type
type EventHandler = Box<dyn Fn(&ExecutionContext) -> RuntimeResult<()> + Send + Sync>;

impl Runtime {
    /// Create a new runtime instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sigmos_runtime::Runtime;
    ///
    /// let runtime = Runtime::new();
    /// ```
    pub fn new() -> Self {
        Self {
            context: Arc::new(RwLock::new(ExecutionContext::default())),
            plugins: HashMap::new(),
            event_handlers: HashMap::new(),
        }
    }

    /// Execute a SIGMOS specification
    ///
    /// # Arguments
    ///
    /// * `spec` - The specification to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sigmos_runtime::Runtime;
    /// use sigmos_core::ast::*;
    ///
    /// # tokio_test::block_on(async {
    /// let mut runtime = Runtime::new();
    /// let spec = Spec {
    ///     name: "Test".to_string(),
    ///     version: Version { major: 1, minor: 0, patch: None },
    ///     description: None,
    ///     inputs: vec![],
    ///     computed: vec![],
    ///     events: vec![],
    ///     constraints: vec![],
    ///     lifecycle: vec![],
    ///     extensions: vec![],
    ///     types: vec![],
    /// };
    ///
    /// runtime.execute(&spec).await.unwrap();
    /// # });
    /// ```
    pub async fn execute(&mut self, spec: &Spec) -> RuntimeResult<()> {
        // Set execution state to running
        {
            let mut context = self.context.write().await;
            context.state = ExecutionState::Running;
        }

        // Execute lifecycle before phase
        self.execute_lifecycle_before(spec).await?;

        // Process input fields
        self.process_inputs(spec).await?;

        // Compute derived fields
        self.compute_fields(spec).await?;

        // Execute lifecycle after phase
        self.execute_lifecycle_after(spec).await?;

        // Set execution state to completed
        {
            let mut context = self.context.write().await;
            context.state = ExecutionState::Completed;
        }

        Ok(())
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin + Send + Sync>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    /// Evaluate an expression in the current context
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sigmos_runtime::Runtime;
    /// use sigmos_core::ast::Expression;
    ///
    /// let runtime = Runtime::new();
    /// let expr = Expression::StringLiteral("Hello World".to_string());
    /// let result = runtime.evaluate_expression(&expr).unwrap();
    /// ```
    pub fn evaluate_expression(&self, expr: &Expression) -> RuntimeResult<JsonValue> {
        self.evaluate_expression_with_context(expr, &HashMap::new())
    }

    /// Evaluate an expression with additional context variables
    pub fn evaluate_expression_with_context(
        &self,
        expr: &Expression,
        context: &HashMap<String, JsonValue>,
    ) -> RuntimeResult<JsonValue> {
        match expr {
            Expression::StringLiteral(s) => Ok(JsonValue::String(s.clone())),
            Expression::Number(n) => Ok(JsonValue::Number(
                serde_json::Number::from_f64(*n)
                    .ok_or_else(|| RuntimeError::Evaluation(format!("Invalid number: {n}")))?,
            )),
            Expression::Boolean(b) => Ok(JsonValue::Bool(*b)),

            Expression::Identifier(name) => {
                // Look up variable in context, then in runtime context
                if let Some(value) = context.get(name) {
                    Ok(value.clone())
                } else {
                    // Try to get from runtime context (async context would require different approach)
                    // For now, return a descriptive placeholder
                    Ok(JsonValue::String(format!("${{{name}}}")))
                }
            }

            Expression::FunctionCall {
                object,
                method,
                arguments,
            } => self.evaluate_function_call(object, method, arguments, context),

            Expression::StringTemplate { parts } => self.evaluate_string_template(parts, context),

            // Arithmetic operators
            Expression::Add(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                let right_val = self.evaluate_expression_with_context(right, context)?;
                self.perform_arithmetic_operation(&left_val, &right_val, ArithmeticOp::Add)
            }
            Expression::Subtract(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                let right_val = self.evaluate_expression_with_context(right, context)?;
                self.perform_arithmetic_operation(&left_val, &right_val, ArithmeticOp::Subtract)
            }
            Expression::Multiply(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                let right_val = self.evaluate_expression_with_context(right, context)?;
                self.perform_arithmetic_operation(&left_val, &right_val, ArithmeticOp::Multiply)
            }
            Expression::Divide(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                let right_val = self.evaluate_expression_with_context(right, context)?;
                self.perform_arithmetic_operation(&left_val, &right_val, ArithmeticOp::Divide)
            }
            Expression::Modulo(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                let right_val = self.evaluate_expression_with_context(right, context)?;
                self.perform_arithmetic_operation(&left_val, &right_val, ArithmeticOp::Modulo)
            }

            // Comparison operators
            Expression::Equal(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                let right_val = self.evaluate_expression_with_context(right, context)?;
                Ok(JsonValue::Bool(self.values_equal(&left_val, &right_val)))
            }
            Expression::NotEqual(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                let right_val = self.evaluate_expression_with_context(right, context)?;
                Ok(JsonValue::Bool(!self.values_equal(&left_val, &right_val)))
            }
            Expression::LessThan(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                let right_val = self.evaluate_expression_with_context(right, context)?;
                self.perform_comparison(&left_val, &right_val, ComparisonOp::LessThan)
            }
            Expression::LessThanOrEqual(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                let right_val = self.evaluate_expression_with_context(right, context)?;
                self.perform_comparison(&left_val, &right_val, ComparisonOp::LessThanOrEqual)
            }
            Expression::GreaterThan(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                let right_val = self.evaluate_expression_with_context(right, context)?;
                self.perform_comparison(&left_val, &right_val, ComparisonOp::GreaterThan)
            }
            Expression::GreaterThanOrEqual(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                let right_val = self.evaluate_expression_with_context(right, context)?;
                self.perform_comparison(&left_val, &right_val, ComparisonOp::GreaterThanOrEqual)
            }

            // Logical operators
            Expression::And(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                if !self.is_truthy(&left_val) {
                    Ok(JsonValue::Bool(false))
                } else {
                    let right_val = self.evaluate_expression_with_context(right, context)?;
                    Ok(JsonValue::Bool(self.is_truthy(&right_val)))
                }
            }
            Expression::Or(left, right) => {
                let left_val = self.evaluate_expression_with_context(left, context)?;
                if self.is_truthy(&left_val) {
                    Ok(JsonValue::Bool(true))
                } else {
                    let right_val = self.evaluate_expression_with_context(right, context)?;
                    Ok(JsonValue::Bool(self.is_truthy(&right_val)))
                }
            }
            Expression::Not(operand) => {
                let val = self.evaluate_expression_with_context(operand, context)?;
                Ok(JsonValue::Bool(!self.is_truthy(&val)))
            }

            // Conditional expression
            Expression::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                let condition_val = self.evaluate_expression_with_context(condition, context)?;
                if self.is_truthy(&condition_val) {
                    self.evaluate_expression_with_context(if_true, context)
                } else {
                    self.evaluate_expression_with_context(if_false, context)
                }
            }

            // Array and object access
            Expression::ArrayAccess(array_expr, index_expr) => {
                let array_val = self.evaluate_expression_with_context(array_expr, context)?;
                let index_val = self.evaluate_expression_with_context(index_expr, context)?;
                self.perform_array_access(&array_val, &index_val)
            }
            Expression::PropertyAccess(object_expr, property) => {
                let object_val = self.evaluate_expression_with_context(object_expr, context)?;
                self.perform_property_access(&object_val, property)
            }
        }
    }

    /// Evaluate a function call
    fn evaluate_function_call(
        &self,
        object: &str,
        method: &str,
        arguments: &[Argument],
        context: &HashMap<String, JsonValue>,
    ) -> RuntimeResult<JsonValue> {
        // Built-in functions
        match (object, method) {
            ("", "len") => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::Evaluation(
                        "len() requires exactly one argument".to_string(),
                    ));
                }
                let arg_value =
                    self.evaluate_expression_with_context(&arguments[0].value, context)?;
                match arg_value {
                    JsonValue::String(s) => {
                        Ok(JsonValue::Number(serde_json::Number::from(s.len())))
                    }
                    JsonValue::Array(arr) => {
                        Ok(JsonValue::Number(serde_json::Number::from(arr.len())))
                    }
                    JsonValue::Object(obj) => {
                        Ok(JsonValue::Number(serde_json::Number::from(obj.len())))
                    }
                    _ => Err(RuntimeError::Evaluation(
                        "len() can only be applied to strings, arrays, or objects".to_string(),
                    )),
                }
            }
            ("", "upper") => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::Evaluation(
                        "upper() requires exactly one argument".to_string(),
                    ));
                }
                let arg_value =
                    self.evaluate_expression_with_context(&arguments[0].value, context)?;
                match arg_value {
                    JsonValue::String(s) => Ok(JsonValue::String(s.to_uppercase())),
                    _ => Err(RuntimeError::Evaluation(
                        "upper() can only be applied to strings".to_string(),
                    )),
                }
            }
            ("", "lower") => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::Evaluation(
                        "lower() requires exactly one argument".to_string(),
                    ));
                }
                let arg_value =
                    self.evaluate_expression_with_context(&arguments[0].value, context)?;
                match arg_value {
                    JsonValue::String(s) => Ok(JsonValue::String(s.to_lowercase())),
                    _ => Err(RuntimeError::Evaluation(
                        "lower() can only be applied to strings".to_string(),
                    )),
                }
            }
            ("", "trim") => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::Evaluation(
                        "trim() requires exactly one argument".to_string(),
                    ));
                }
                let arg_value =
                    self.evaluate_expression_with_context(&arguments[0].value, context)?;
                match arg_value {
                    JsonValue::String(s) => Ok(JsonValue::String(s.trim().to_string())),
                    _ => Err(RuntimeError::Evaluation(
                        "trim() can only be applied to strings".to_string(),
                    )),
                }
            }
            ("", "abs") => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::Evaluation(
                        "abs() requires exactly one argument".to_string(),
                    ));
                }
                let arg_value =
                    self.evaluate_expression_with_context(&arguments[0].value, context)?;
                match arg_value {
                    JsonValue::Number(n) => {
                        let f = n.as_f64().ok_or_else(|| {
                            RuntimeError::Evaluation("Invalid number for abs()".to_string())
                        })?;
                        Ok(JsonValue::Number(
                            serde_json::Number::from_f64(f.abs()).ok_or_else(|| {
                                RuntimeError::Evaluation(
                                    "Result of abs() is not a valid number".to_string(),
                                )
                            })?,
                        ))
                    }
                    _ => Err(RuntimeError::Evaluation(
                        "abs() can only be applied to numbers".to_string(),
                    )),
                }
            }
            // Plugin method calls
            (plugin_name, method_name) if !plugin_name.is_empty() => {
                if let Some(plugin) = self.plugins.get(plugin_name) {
                    // Convert arguments to HashMap
                    let mut args = HashMap::new();
                    for (i, arg) in arguments.iter().enumerate() {
                        let arg_name = if arg.name.is_empty() {
                            format!("arg_{i}")
                        } else {
                            arg.name.clone()
                        };
                        let arg_value =
                            self.evaluate_expression_with_context(&arg.value, context)?;
                        args.insert(arg_name, arg_value);
                    }

                    plugin.execute(method_name, &args)
                } else {
                    Err(RuntimeError::Evaluation(format!(
                        "Plugin '{plugin_name}' not found"
                    )))
                }
            }
            // Unknown function
            _ => {
                let function_name = if object.is_empty() {
                    method.to_string()
                } else {
                    format!("{object}.{method}")
                };
                Err(RuntimeError::Evaluation(format!(
                    "Unknown function: {function_name}"
                )))
            }
        }
    }

    /// Evaluate a string template
    fn evaluate_string_template(
        &self,
        parts: &[TemplatePart],
        context: &HashMap<String, JsonValue>,
    ) -> RuntimeResult<JsonValue> {
        let mut result = String::new();

        for part in parts {
            match part {
                TemplatePart::Text(text) => {
                    result.push_str(text);
                }
                TemplatePart::Variable(var_name) => {
                    if let Some(value) = context.get(var_name) {
                        match value {
                            JsonValue::String(s) => result.push_str(s),
                            JsonValue::Number(n) => result.push_str(&n.to_string()),
                            JsonValue::Bool(b) => result.push_str(&b.to_string()),
                            JsonValue::Null => result.push_str("null"),
                            _ => result.push_str(&value.to_string()),
                        }
                    } else {
                        // Variable not found, include placeholder
                        result.push_str(&format!("${{{var_name}}}"));
                    }
                }
            }
        }

        Ok(JsonValue::String(result))
    }

    /// Perform arithmetic operations
    fn perform_arithmetic_operation(
        &self,
        left: &JsonValue,
        right: &JsonValue,
        op: ArithmeticOp,
    ) -> RuntimeResult<JsonValue> {
        match (left, right) {
            (JsonValue::Number(l), JsonValue::Number(r)) => {
                let l_f64 = l.as_f64().ok_or_else(|| {
                    RuntimeError::Evaluation(
                        "Invalid left operand for arithmetic operation".to_string(),
                    )
                })?;
                let r_f64 = r.as_f64().ok_or_else(|| {
                    RuntimeError::Evaluation(
                        "Invalid right operand for arithmetic operation".to_string(),
                    )
                })?;

                let result = match op {
                    ArithmeticOp::Add => l_f64 + r_f64,
                    ArithmeticOp::Subtract => l_f64 - r_f64,
                    ArithmeticOp::Multiply => l_f64 * r_f64,
                    ArithmeticOp::Divide => {
                        if r_f64 == 0.0 {
                            return Err(RuntimeError::Evaluation("Division by zero".to_string()));
                        }
                        l_f64 / r_f64
                    }
                    ArithmeticOp::Modulo => {
                        if r_f64 == 0.0 {
                            return Err(RuntimeError::Evaluation("Modulo by zero".to_string()));
                        }
                        l_f64 % r_f64
                    }
                };

                Ok(JsonValue::Number(
                    serde_json::Number::from_f64(result).ok_or_else(|| {
                        RuntimeError::Evaluation(
                            "Arithmetic operation result is not a valid number".to_string(),
                        )
                    })?,
                ))
            }
            // String concatenation for addition
            (JsonValue::String(l), JsonValue::String(r)) if matches!(op, ArithmeticOp::Add) => {
                Ok(JsonValue::String(format!("{l}{r}")))
            }
            _ => Err(RuntimeError::Evaluation(format!(
                "Cannot perform arithmetic operation on {left:?} and {right:?}"
            ))),
        }
    }

    /// Perform comparison operations
    fn perform_comparison(
        &self,
        left: &JsonValue,
        right: &JsonValue,
        op: ComparisonOp,
    ) -> RuntimeResult<JsonValue> {
        match (left, right) {
            (JsonValue::Number(l), JsonValue::Number(r)) => {
                let l_f64 = l.as_f64().ok_or_else(|| {
                    RuntimeError::Evaluation("Invalid left operand for comparison".to_string())
                })?;
                let r_f64 = r.as_f64().ok_or_else(|| {
                    RuntimeError::Evaluation("Invalid right operand for comparison".to_string())
                })?;

                let result = match op {
                    ComparisonOp::LessThan => l_f64 < r_f64,
                    ComparisonOp::LessThanOrEqual => l_f64 <= r_f64,
                    ComparisonOp::GreaterThan => l_f64 > r_f64,
                    ComparisonOp::GreaterThanOrEqual => l_f64 >= r_f64,
                };

                Ok(JsonValue::Bool(result))
            }
            (JsonValue::String(l), JsonValue::String(r)) => {
                let result = match op {
                    ComparisonOp::LessThan => l < r,
                    ComparisonOp::LessThanOrEqual => l <= r,
                    ComparisonOp::GreaterThan => l > r,
                    ComparisonOp::GreaterThanOrEqual => l >= r,
                };

                Ok(JsonValue::Bool(result))
            }
            _ => Err(RuntimeError::Evaluation(format!(
                "Cannot compare {left:?} and {right:?}"
            ))),
        }
    }

    /// Check if two values are equal
    #[allow(clippy::only_used_in_recursion)]
    fn values_equal(&self, left: &JsonValue, right: &JsonValue) -> bool {
        match (left, right) {
            (JsonValue::Null, JsonValue::Null) => true,
            (JsonValue::Bool(l), JsonValue::Bool(r)) => l == r,
            (JsonValue::Number(l), JsonValue::Number(r)) => {
                if let (Some(l_f64), Some(r_f64)) = (l.as_f64(), r.as_f64()) {
                    (l_f64 - r_f64).abs() < f64::EPSILON
                } else {
                    false
                }
            }
            (JsonValue::String(l), JsonValue::String(r)) => l == r,
            (JsonValue::Array(l), JsonValue::Array(r)) => {
                l.len() == r.len() && l.iter().zip(r.iter()).all(|(a, b)| self.values_equal(a, b))
            }
            (JsonValue::Object(l), JsonValue::Object(r)) => {
                l.len() == r.len()
                    && l.iter()
                        .all(|(k, v)| r.get(k).is_some_and(|rv| self.values_equal(v, rv)))
            }
            _ => false,
        }
    }

    /// Check if a value is truthy
    fn is_truthy(&self, value: &JsonValue) -> bool {
        match value {
            JsonValue::Null => false,
            JsonValue::Bool(b) => *b,
            JsonValue::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
            JsonValue::String(s) => !s.is_empty(),
            JsonValue::Array(arr) => !arr.is_empty(),
            JsonValue::Object(obj) => !obj.is_empty(),
        }
    }

    /// Perform array access
    fn perform_array_access(
        &self,
        array: &JsonValue,
        index: &JsonValue,
    ) -> RuntimeResult<JsonValue> {
        match (array, index) {
            (JsonValue::Array(arr), JsonValue::Number(n)) => {
                let idx = n.as_u64().ok_or_else(|| {
                    RuntimeError::Evaluation(
                        "Array index must be a non-negative integer".to_string(),
                    )
                })? as usize;

                arr.get(idx).cloned().ok_or_else(|| {
                    RuntimeError::Evaluation(format!("Array index {idx} out of bounds"))
                })
            }
            (JsonValue::Object(obj), JsonValue::String(key)) => {
                Ok(obj.get(key).cloned().unwrap_or(JsonValue::Null))
            }
            _ => Err(RuntimeError::Evaluation(
                "Invalid array/object access".to_string(),
            )),
        }
    }

    /// Perform property access
    fn perform_property_access(
        &self,
        object: &JsonValue,
        property: &str,
    ) -> RuntimeResult<JsonValue> {
        match object {
            JsonValue::Object(obj) => Ok(obj.get(property).cloned().unwrap_or(JsonValue::Null)),
            _ => Err(RuntimeError::Evaluation(
                "Property access can only be performed on objects".to_string(),
            )),
        }
    }

    /// Process input fields
    async fn process_inputs(&self, spec: &Spec) -> RuntimeResult<()> {
        let mut context = self.context.write().await;

        for field in &spec.inputs {
            let mut field_value = serde_json::Value::Null;

            // Process field modifiers
            for modifier in &field.modifiers {
                match modifier {
                    Modifier::Default(expr) => {
                        // Evaluate default expression
                        field_value = self.evaluate_expression(expr)?;
                    }
                    Modifier::Optional => {
                        // Optional fields can remain null
                    }
                    Modifier::Secret => {
                        // Mark as secret (affects logging/serialization)
                        // For now, just process normally
                    }
                    Modifier::Generate => {
                        // Generate a value based on type
                        field_value = self.generate_value_for_type(&field.type_expr)?;
                    }
                    _ => {
                        // Other modifiers don't affect initial value
                    }
                }
            }

            context.variables.insert(field.name.clone(), field_value);
        }

        Ok(())
    }

    /// Compute derived fields
    async fn compute_fields(&self, spec: &Spec) -> RuntimeResult<()> {
        let context_read = self.context.read().await;
        let variable_context = context_read.variables.clone();
        drop(context_read);

        let mut computed_values = HashMap::new();

        for computed in &spec.computed {
            // Evaluate the computed expression with current variable context
            let computed_value =
                self.evaluate_expression_with_context(&computed.expression, &variable_context)?;

            computed_values.insert(computed.name.clone(), computed_value);
        }

        // Update the computed cache
        let mut context = self.context.write().await;
        for (name, value) in computed_values {
            context.computed_cache.insert(name, value);
        }

        Ok(())
    }

    /// Execute lifecycle before phase
    async fn execute_lifecycle_before(&self, spec: &Spec) -> RuntimeResult<()> {
        for lifecycle in &spec.lifecycle {
            if matches!(lifecycle.phase, LifecyclePhase::Before) {
                // Execute the lifecycle action
                match &lifecycle.action {
                    Action::FunctionCall {
                        object,
                        method,
                        arguments,
                    } => {
                        let context_read = self.context.read().await;
                        let variable_context = context_read.variables.clone();
                        drop(context_read);

                        self.evaluate_function_call(object, method, arguments, &variable_context)?;
                    }
                    Action::Identifier(name) => {
                        // Execute identifier as a function call with no arguments
                        let context_read = self.context.read().await;
                        let variable_context = context_read.variables.clone();
                        drop(context_read);

                        // For now, treat identifier as a simple function call
                        let empty_args = vec![];
                        self.evaluate_function_call(
                            "builtin",
                            name,
                            &empty_args,
                            &variable_context,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Execute lifecycle after phase
    async fn execute_lifecycle_after(&self, spec: &Spec) -> RuntimeResult<()> {
        for lifecycle in &spec.lifecycle {
            if matches!(lifecycle.phase, LifecyclePhase::After) {
                // Execute the lifecycle action
                match &lifecycle.action {
                    Action::FunctionCall {
                        object,
                        method,
                        arguments,
                    } => {
                        let context_read = self.context.read().await;
                        let variable_context = context_read.variables.clone();
                        drop(context_read);

                        self.evaluate_function_call(object, method, arguments, &variable_context)?;
                    }
                    Action::Identifier(name) => {
                        // Execute identifier as a function call with no arguments
                        let context_read = self.context.read().await;
                        let variable_context = context_read.variables.clone();
                        drop(context_read);

                        // For now, treat identifier as a simple function call
                        let empty_args = vec![];
                        self.evaluate_function_call(
                            "builtin",
                            name,
                            &empty_args,
                            &variable_context,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Generate a default value for a given type
    fn generate_value_for_type(&self, type_expr: &TypeExpr) -> RuntimeResult<JsonValue> {
        match type_expr {
            TypeExpr::Primitive(PrimitiveType::String) => {
                Ok(JsonValue::String("generated".to_string()))
            }
            TypeExpr::Primitive(PrimitiveType::Int) => {
                Ok(JsonValue::Number(serde_json::Number::from(0)))
            }
            TypeExpr::Primitive(PrimitiveType::Float) => Ok(JsonValue::Number(
                serde_json::Number::from_f64(0.0).unwrap(),
            )),
            TypeExpr::Primitive(PrimitiveType::Bool) => Ok(JsonValue::Bool(false)),
            TypeExpr::Primitive(PrimitiveType::Null) => Ok(JsonValue::Null),
            TypeExpr::Generic { name, args: _ } => match name.as_str() {
                "Array" => Ok(JsonValue::Array(vec![])),
                "Map" => Ok(JsonValue::Object(serde_json::Map::new())),
                _ => Ok(JsonValue::Null),
            },
            TypeExpr::Reference(_) => {
                // For user-defined types, generate null for now
                Ok(JsonValue::Null)
            }
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sigmos_core::ast::{Spec, Field, TypeExpr, PrimitiveType};

    #[tokio::test]
    async fn test_runtime_creation() {
        let runtime = Runtime::new();
        assert!(runtime.plugins.is_empty());
    }

    #[tokio::test]
    async fn test_execute_empty_spec() {
        let mut runtime = Runtime::new();
        let spec = Spec {
            name: "Test".to_string(),
            version: Version {
                major: 1,
                minor: 0,
                patch: None,
            },
            description: None,
            inputs: vec![],
            computed: vec![],
            events: vec![],
            constraints: vec![],
            lifecycle: vec![],
            extensions: vec![],
            types: vec![],
        };

        runtime.execute(&spec).await.unwrap();
    }

    #[test]
    fn test_enhanced_arithmetic_expressions() {
        let runtime = Runtime::new();

        // Test addition
        let add_expr = Expression::Add(
            Box::new(Expression::Number(5.0)),
            Box::new(Expression::Number(3.0)),
        );
        let result = runtime.evaluate_expression(&add_expr).unwrap();
        assert_eq!(
            result,
            serde_json::Value::Number(serde_json::Number::from_f64(8.0).unwrap())
        );

        // Test string concatenation
        let concat_expr = Expression::Add(
            Box::new(Expression::StringLiteral("Hello ".to_string())),
            Box::new(Expression::StringLiteral("World".to_string())),
        );
        let result = runtime.evaluate_expression(&concat_expr).unwrap();
        assert_eq!(result, serde_json::Value::String("Hello World".to_string()));

        // Test subtraction
        let sub_expr = Expression::Subtract(
            Box::new(Expression::Number(10.0)),
            Box::new(Expression::Number(4.0)),
        );
        let result = runtime.evaluate_expression(&sub_expr).unwrap();
        assert_eq!(
            result,
            serde_json::Value::Number(serde_json::Number::from_f64(6.0).unwrap())
        );

        // Test multiplication
        let mul_expr = Expression::Multiply(
            Box::new(Expression::Number(3.0)),
            Box::new(Expression::Number(4.0)),
        );
        let result = runtime.evaluate_expression(&mul_expr).unwrap();
        assert_eq!(
            result,
            serde_json::Value::Number(serde_json::Number::from_f64(12.0).unwrap())
        );

        // Test division
        let div_expr = Expression::Divide(
            Box::new(Expression::Number(15.0)),
            Box::new(Expression::Number(3.0)),
        );
        let result = runtime.evaluate_expression(&div_expr).unwrap();
        assert_eq!(
            result,
            serde_json::Value::Number(serde_json::Number::from_f64(5.0).unwrap())
        );

        // Test modulo
        let mod_expr = Expression::Modulo(
            Box::new(Expression::Number(17.0)),
            Box::new(Expression::Number(5.0)),
        );
        let result = runtime.evaluate_expression(&mod_expr).unwrap();
        assert_eq!(
            result,
            serde_json::Value::Number(serde_json::Number::from_f64(2.0).unwrap())
        );
    }

    #[test]
    fn test_enhanced_comparison_expressions() {
        let runtime = Runtime::new();

        // Test less than
        let lt_expr = Expression::LessThan(
            Box::new(Expression::Number(3.0)),
            Box::new(Expression::Number(5.0)),
        );
        let result = runtime.evaluate_expression(&lt_expr).unwrap();
        assert_eq!(result, serde_json::Value::Bool(true));

        // Test greater than
        let gt_expr = Expression::GreaterThan(
            Box::new(Expression::Number(7.0)),
            Box::new(Expression::Number(4.0)),
        );
        let result = runtime.evaluate_expression(&gt_expr).unwrap();
        assert_eq!(result, serde_json::Value::Bool(true));

        // Test equal
        let eq_expr = Expression::Equal(
            Box::new(Expression::StringLiteral("test".to_string())),
            Box::new(Expression::StringLiteral("test".to_string())),
        );
        let result = runtime.evaluate_expression(&eq_expr).unwrap();
        assert_eq!(result, serde_json::Value::Bool(true));

        // Test not equal
        let ne_expr = Expression::NotEqual(
            Box::new(Expression::Number(5.0)),
            Box::new(Expression::Number(3.0)),
        );
        let result = runtime.evaluate_expression(&ne_expr).unwrap();
        assert_eq!(result, serde_json::Value::Bool(true));
    }

    #[test]
    fn test_enhanced_logical_expressions() {
        let runtime = Runtime::new();

        // Test AND - both true
        let and_expr = Expression::And(
            Box::new(Expression::Boolean(true)),
            Box::new(Expression::Boolean(true)),
        );
        let result = runtime.evaluate_expression(&and_expr).unwrap();
        assert_eq!(result, serde_json::Value::Bool(true));

        // Test AND - one false
        let and_false_expr = Expression::And(
            Box::new(Expression::Boolean(true)),
            Box::new(Expression::Boolean(false)),
        );
        let result = runtime.evaluate_expression(&and_false_expr).unwrap();
        assert_eq!(result, serde_json::Value::Bool(false));

        // Test OR - one true
        let or_expr = Expression::Or(
            Box::new(Expression::Boolean(false)),
            Box::new(Expression::Boolean(true)),
        );
        let result = runtime.evaluate_expression(&or_expr).unwrap();
        assert_eq!(result, serde_json::Value::Bool(true));

        // Test NOT
        let not_expr = Expression::Not(Box::new(Expression::Boolean(false)));
        let result = runtime.evaluate_expression(&not_expr).unwrap();
        assert_eq!(result, serde_json::Value::Bool(true));
    }

    #[test]
    fn test_enhanced_conditional_expressions() {
        let runtime = Runtime::new();

        // Test conditional - true condition
        let cond_true_expr = Expression::Conditional {
            condition: Box::new(Expression::Boolean(true)),
            if_true: Box::new(Expression::StringLiteral("yes".to_string())),
            if_false: Box::new(Expression::StringLiteral("no".to_string())),
        };
        let result = runtime.evaluate_expression(&cond_true_expr).unwrap();
        assert_eq!(result, serde_json::Value::String("yes".to_string()));

        // Test conditional - false condition
        let cond_false_expr = Expression::Conditional {
            condition: Box::new(Expression::Boolean(false)),
            if_true: Box::new(Expression::Number(1.0)),
            if_false: Box::new(Expression::Number(2.0)),
        };
        let result = runtime.evaluate_expression(&cond_false_expr).unwrap();
        assert_eq!(
            result,
            serde_json::Value::Number(serde_json::Number::from_f64(2.0).unwrap())
        );
    }

    #[test]
    fn test_enhanced_builtin_functions() {
        let runtime = Runtime::new();

        // Test len() function
        let len_expr = Expression::FunctionCall {
            object: "".to_string(),
            method: "len".to_string(),
            arguments: vec![Argument {
                name: "".to_string(),
                value: Expression::StringLiteral("hello".to_string()),
            }],
        };
        let result = runtime.evaluate_expression(&len_expr).unwrap();
        assert_eq!(
            result,
            serde_json::Value::Number(serde_json::Number::from(5))
        );

        // Test upper() function
        let upper_expr = Expression::FunctionCall {
            object: "".to_string(),
            method: "upper".to_string(),
            arguments: vec![Argument {
                name: "".to_string(),
                value: Expression::StringLiteral("hello".to_string()),
            }],
        };
        let result = runtime.evaluate_expression(&upper_expr).unwrap();
        assert_eq!(result, serde_json::Value::String("HELLO".to_string()));

        // Test lower() function
        let lower_expr = Expression::FunctionCall {
            object: "".to_string(),
            method: "lower".to_string(),
            arguments: vec![Argument {
                name: "".to_string(),
                value: Expression::StringLiteral("WORLD".to_string()),
            }],
        };
        let result = runtime.evaluate_expression(&lower_expr).unwrap();
        assert_eq!(result, serde_json::Value::String("world".to_string()));

        // Test trim() function
        let trim_expr = Expression::FunctionCall {
            object: "".to_string(),
            method: "trim".to_string(),
            arguments: vec![Argument {
                name: "".to_string(),
                value: Expression::StringLiteral("  test  ".to_string()),
            }],
        };
        let result = runtime.evaluate_expression(&trim_expr).unwrap();
        assert_eq!(result, serde_json::Value::String("test".to_string()));

        // Test abs() function
        let abs_expr = Expression::FunctionCall {
            object: "".to_string(),
            method: "abs".to_string(),
            arguments: vec![Argument {
                name: "".to_string(),
                value: Expression::Number(-5.5),
            }],
        };
        let result = runtime.evaluate_expression(&abs_expr).unwrap();
        assert_eq!(
            result,
            serde_json::Value::Number(serde_json::Number::from_f64(5.5).unwrap())
        );
    }

    #[test]
    fn test_enhanced_string_templates() {
        let runtime = Runtime::new();
        let mut context = std::collections::HashMap::new();
        context.insert(
            "name".to_string(),
            serde_json::Value::String("Alice".to_string()),
        );
        context.insert(
            "age".to_string(),
            serde_json::Value::Number(serde_json::Number::from(30)),
        );

        // Test string template with variables
        let template_expr = Expression::StringTemplate {
            parts: vec![
                TemplatePart::Text("Hello, ".to_string()),
                TemplatePart::Variable("name".to_string()),
                TemplatePart::Text("! You are ".to_string()),
                TemplatePart::Variable("age".to_string()),
                TemplatePart::Text(" years old.".to_string()),
            ],
        };
        let result = runtime
            .evaluate_expression_with_context(&template_expr, &context)
            .unwrap();
        assert_eq!(
            result,
            serde_json::Value::String("Hello, Alice! You are 30 years old.".to_string())
        );

        // Test template with missing variable
        let template_missing_expr = Expression::StringTemplate {
            parts: vec![
                TemplatePart::Text("Hello, ".to_string()),
                TemplatePart::Variable("unknown".to_string()),
                TemplatePart::Text("!".to_string()),
            ],
        };
        let result = runtime
            .evaluate_expression_with_context(&template_missing_expr, &context)
            .unwrap();
        assert_eq!(
            result,
            serde_json::Value::String("Hello, ${unknown}!".to_string())
        );
    }

    #[test]
    fn test_enhanced_variable_resolution() {
        let runtime = Runtime::new();
        let mut context = std::collections::HashMap::new();
        context.insert(
            "x".to_string(),
            serde_json::Value::Number(serde_json::Number::from(42)),
        );
        context.insert(
            "greeting".to_string(),
            serde_json::Value::String("Hello".to_string()),
        );

        // Test variable resolution
        let var_expr = Expression::Identifier("x".to_string());
        let result = runtime
            .evaluate_expression_with_context(&var_expr, &context)
            .unwrap();
        assert_eq!(
            result,
            serde_json::Value::Number(serde_json::Number::from(42))
        );

        // Test string variable
        let str_var_expr = Expression::Identifier("greeting".to_string());
        let result = runtime
            .evaluate_expression_with_context(&str_var_expr, &context)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("Hello".to_string()));

        // Test unknown variable (returns placeholder)
        let unknown_expr = Expression::Identifier("unknown".to_string());
        let result = runtime
            .evaluate_expression_with_context(&unknown_expr, &context)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("${unknown}".to_string()));
    }

    #[test]
    fn test_enhanced_complex_expressions() {
        let runtime = Runtime::new();
        let mut context = std::collections::HashMap::new();
        context.insert(
            "a".to_string(),
            serde_json::Value::Number(serde_json::Number::from(10)),
        );
        context.insert(
            "b".to_string(),
            serde_json::Value::Number(serde_json::Number::from(5)),
        );

        // Test complex arithmetic: (a + b) * 2
        let complex_expr = Expression::Multiply(
            Box::new(Expression::Add(
                Box::new(Expression::Identifier("a".to_string())),
                Box::new(Expression::Identifier("b".to_string())),
            )),
            Box::new(Expression::Number(2.0)),
        );
        let result = runtime
            .evaluate_expression_with_context(&complex_expr, &context)
            .unwrap();
        assert_eq!(
            result,
            serde_json::Value::Number(serde_json::Number::from_f64(30.0).unwrap())
        );

        // Test complex conditional: a > b ? "greater" : "not greater"
        let complex_cond = Expression::Conditional {
            condition: Box::new(Expression::GreaterThan(
                Box::new(Expression::Identifier("a".to_string())),
                Box::new(Expression::Identifier("b".to_string())),
            )),
            if_true: Box::new(Expression::StringLiteral("greater".to_string())),
            if_false: Box::new(Expression::StringLiteral("not greater".to_string())),
        };
        let result = runtime
            .evaluate_expression_with_context(&complex_cond, &context)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("greater".to_string()));
    }

    #[test]
    fn test_enhanced_error_handling() {
        let runtime = Runtime::new();

        // Test division by zero
        let div_zero_expr = Expression::Divide(
            Box::new(Expression::Number(10.0)),
            Box::new(Expression::Number(0.0)),
        );
        let result = runtime.evaluate_expression(&div_zero_expr);
        assert!(result.is_err());

        // Test modulo by zero
        let mod_zero_expr = Expression::Modulo(
            Box::new(Expression::Number(10.0)),
            Box::new(Expression::Number(0.0)),
        );
        let result = runtime.evaluate_expression(&mod_zero_expr);
        assert!(result.is_err());

        // Test invalid function
        let invalid_func_expr = Expression::FunctionCall {
            object: "".to_string(),
            method: "nonexistent".to_string(),
            arguments: vec![],
        };
        let result = runtime.evaluate_expression(&invalid_func_expr);
        assert!(result.is_err());

        // Test len() with wrong argument type
        let invalid_len_expr = Expression::FunctionCall {
            object: "".to_string(),
            method: "len".to_string(),
            arguments: vec![Argument {
                name: "".to_string(),
                value: Expression::Number(42.0),
            }],
        };
        let result = runtime.evaluate_expression(&invalid_len_expr);
        assert!(result.is_err());
    }
}

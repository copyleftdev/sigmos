//! # Abstract Syntax Tree (AST) for SIGMOS
//!
//! This module defines the AST nodes for SIGMOS specifications.
//! All AST nodes are designed to be serializable and support rich error reporting.
//!
//! # Examples
//!
//! ```rust
//! use sigmos_core::ast::{Spec, Version};
//!
//! let spec = Spec {
//!     name: "Agent".to_string(),
//!     version: Version { major: 1, minor: 0, patch: None },
//!     description: None,
//!     inputs: vec![],
//!     computed: vec![],
//!     events: vec![],
//!     constraints: vec![],
//!     lifecycle: vec![],
//!     extensions: vec![],
//!     types: vec![],
//! };
//! ```

use serde::{Deserialize, Serialize};

/// Version specification for SIGMOS specs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: Option<u32>,
}

/// Root SIGMOS specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    pub name: String,
    pub version: Version,
    pub description: Option<String>,
    pub inputs: Vec<FieldDef>,
    pub computed: Vec<ComputedField>,
    pub events: Vec<EventDef>,
    pub constraints: Vec<ConstraintDef>,
    pub lifecycle: Vec<LifecycleDef>,
    pub extensions: Vec<ExtensionDef>,
    pub types: Vec<TypeDef>,
}

/// Field definition with type and modifiers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub type_expr: TypeExpr,
    pub modifiers: Vec<Modifier>,
}

/// Type expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExpr {
    Primitive(PrimitiveType),
    Generic {
        name: String,
        args: Vec<TypeExpr>,
    },
    Reference(String),
}

/// Primitive types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveType {
    String,
    Int,
    Float,
    Bool,
    Null,
}

/// Field modifiers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Modifier {
    Optional,
    Readonly,
    Default(Expression),
    Computed,
    Secret,
    Generate,
    Ref(String),
}

/// Computed field with expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputedField {
    pub name: String,
    pub expression: Expression,
}

/// Event definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventDef {
    pub event_type: EventType,
    pub parameter: String,
    pub action: Action,
}

/// Event types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventType {
    OnCreate,
    OnChange,
    OnError,
    Custom(String),
}

/// Actions that can be triggered
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    FunctionCall {
        object: String,
        method: String,
        arguments: Vec<Argument>,
    },
    Identifier(String),
}

/// Function call arguments
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Argument {
    pub name: String,
    pub value: Expression,
}

/// Expressions in SIGMOS
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    StringLiteral(String),
    StringTemplate {
        parts: Vec<TemplatePart>,
    },
    Number(f64),
    Boolean(bool),
    Identifier(String),
    FunctionCall {
        object: String,
        method: String,
        arguments: Vec<Argument>,
    },
}

/// Parts of string templates
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemplatePart {
    Text(String),
    Variable(String),
}

/// Constraint definitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintDef {
    pub constraint_type: ConstraintType,
    pub expression: Expression,
}

/// Constraint types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintType {
    Assert,
    Ensure,
}

/// Lifecycle definitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecycleDef {
    pub phase: LifecyclePhase,
    pub action: Action,
}

/// Lifecycle phases
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifecyclePhase {
    Before,
    After,
    Finally,
}

/// Extension definitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionDef {
    pub name: String,
    pub import_spec: String,
}

/// Type definitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: String,
    pub type_expr: TypeExpr,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.patch {
            Some(patch) => write!(f, "{}.{}.{}", self.major, self.minor, patch),
            None => write!(f, "{}.{}", self.major, self.minor),
        }
    }
}

impl std::fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::String => write!(f, "string"),
            PrimitiveType::Int => write!(f, "int"),
            PrimitiveType::Float => write!(f, "float"),
            PrimitiveType::Bool => write!(f, "bool"),
            PrimitiveType::Null => write!(f, "null"),
        }
    }
}

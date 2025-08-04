//! # SIGMOS Type System
//!
//! This module defines the type system for SIGMOS, including type checking,
//! validation, and constraint resolution.
//!
//! # Examples
//!
//! ```rust
//! use sigmos_core::types::{TypeChecker, TypeContext};
//! use sigmos_core::ast::{TypeExpr, PrimitiveType};
//!
//! let mut checker = TypeChecker::new();
//! let string_type = TypeExpr::Primitive(PrimitiveType::String);
//! assert!(checker.is_valid_type(&string_type));
//! ```

use crate::ast::{TypeExpr, PrimitiveType, FieldDef, Spec};
use crate::{ParseError, ParseResult};
use std::collections::HashMap;

/// Type checker for SIGMOS specifications
#[derive(Debug, Default)]
pub struct TypeChecker {
    /// User-defined types
    user_types: HashMap<String, TypeExpr>,
    /// Built-in type registry
    builtin_types: HashMap<String, TypeExpr>,
}

/// Type checking context
#[derive(Debug, Default)]
pub struct TypeContext {
    /// Available variables in scope
    variables: HashMap<String, TypeExpr>,
    /// Function signatures
    functions: HashMap<String, FunctionSignature>,
}

/// Function signature for type checking
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub parameters: Vec<(String, TypeExpr)>,
    pub return_type: TypeExpr,
}

impl TypeChecker {
    /// Create a new type checker with built-in types
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sigmos_core::types::TypeChecker;
    ///
    /// let checker = TypeChecker::new();
    /// ```
    pub fn new() -> Self {
        let mut checker = Self::default();
        checker.register_builtin_types();
        checker
    }

    /// Register built-in types
    fn register_builtin_types(&mut self) {
        self.builtin_types.insert(
            "list".to_string(),
            TypeExpr::Generic {
                name: "list".to_string(),
                args: vec![TypeExpr::Primitive(PrimitiveType::String)], // placeholder
            },
        );
        self.builtin_types.insert(
            "map".to_string(),
            TypeExpr::Generic {
                name: "map".to_string(),
                args: vec![
                    TypeExpr::Primitive(PrimitiveType::String),
                    TypeExpr::Primitive(PrimitiveType::String),
                ],
            },
        );
    }

    /// Check if a type expression is valid
    ///
    /// # Arguments
    ///
    /// * `type_expr` - The type expression to validate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sigmos_core::types::TypeChecker;
    /// use sigmos_core::ast::{TypeExpr, PrimitiveType};
    ///
    /// let checker = TypeChecker::new();
    /// let string_type = TypeExpr::Primitive(PrimitiveType::String);
    /// assert!(checker.is_valid_type(&string_type));
    /// ```
    pub fn is_valid_type(&self, type_expr: &TypeExpr) -> bool {
        match type_expr {
            TypeExpr::Primitive(_) => true,
            TypeExpr::Reference(name) => {
                self.user_types.contains_key(name) || self.builtin_types.contains_key(name)
            }
            TypeExpr::Generic { name, args } => {
                if !self.builtin_types.contains_key(name) {
                    return false;
                }
                args.iter().all(|arg| self.is_valid_type(arg))
            }
        }
    }

    /// Register a user-defined type
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the type
    /// * `type_expr` - The type definition
    pub fn register_type(&mut self, name: String, type_expr: TypeExpr) -> ParseResult<()> {
        if self.builtin_types.contains_key(&name) {
            return Err(ParseError::Type(format!(
                "Cannot redefine built-in type: {}",
                name
            )));
        }

        if !self.is_valid_type(&type_expr) {
            return Err(ParseError::Type(format!(
                "Invalid type definition for: {}",
                name
            )));
        }

        self.user_types.insert(name, type_expr);
        Ok(())
    }

    /// Validate a complete SIGMOS specification
    ///
    /// # Arguments
    ///
    /// * `spec` - The specification to validate
    pub fn validate_spec(&mut self, spec: &Spec) -> ParseResult<()> {
        // Register user-defined types first
        for type_def in &spec.types {
            self.register_type(type_def.name.clone(), type_def.type_expr.clone())?;
        }

        // Validate input fields
        for field in &spec.inputs {
            self.validate_field(field)?;
        }

        // Validate computed fields
        for computed in &spec.computed {
            // TODO: Validate expressions
            self.validate_computed_field(computed)?;
        }

        Ok(())
    }

    /// Validate a field definition
    fn validate_field(&self, field: &FieldDef) -> ParseResult<()> {
        if !self.is_valid_type(&field.type_expr) {
            return Err(ParseError::Type(format!(
                "Invalid type for field '{}': {:?}",
                field.name, field.type_expr
            )));
        }

        // TODO: Validate modifiers
        Ok(())
    }

    /// Validate a computed field
    fn validate_computed_field(&self, _computed: &crate::ast::ComputedField) -> ParseResult<()> {
        // TODO: Implement expression type checking
        Ok(())
    }

    /// Get the type of an expression in a given context
    pub fn type_of_expression(
        &self,
        _expr: &crate::ast::Expression,
        _context: &TypeContext,
    ) -> ParseResult<TypeExpr> {
        // TODO: Implement expression type inference
        Ok(TypeExpr::Primitive(PrimitiveType::String))
    }
}

impl TypeContext {
    /// Create a new empty type context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a variable to the context
    pub fn add_variable(&mut self, name: String, type_expr: TypeExpr) {
        self.variables.insert(name, type_expr);
    }

    /// Get the type of a variable
    pub fn get_variable_type(&self, name: &str) -> Option<&TypeExpr> {
        self.variables.get(name)
    }

    /// Add a function signature to the context
    pub fn add_function(&mut self, name: String, signature: FunctionSignature) {
        self.functions.insert(name, signature);
    }

    /// Get a function signature
    pub fn get_function(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_primitive_types_are_valid() {
        let checker = TypeChecker::new();
        
        assert!(checker.is_valid_type(&TypeExpr::Primitive(PrimitiveType::String)));
        assert!(checker.is_valid_type(&TypeExpr::Primitive(PrimitiveType::Int)));
        assert!(checker.is_valid_type(&TypeExpr::Primitive(PrimitiveType::Float)));
        assert!(checker.is_valid_type(&TypeExpr::Primitive(PrimitiveType::Bool)));
        assert!(checker.is_valid_type(&TypeExpr::Primitive(PrimitiveType::Null)));
    }

    #[test]
    fn test_builtin_generic_types() {
        let checker = TypeChecker::new();
        
        let list_type = TypeExpr::Generic {
            name: "list".to_string(),
            args: vec![TypeExpr::Primitive(PrimitiveType::String)],
        };
        assert!(checker.is_valid_type(&list_type));
    }

    #[test]
    fn test_user_type_registration() {
        let mut checker = TypeChecker::new();
        
        let user_type = TypeExpr::Primitive(PrimitiveType::String);
        checker.register_type("UserId".to_string(), user_type).unwrap();
        
        let reference = TypeExpr::Reference("UserId".to_string());
        assert!(checker.is_valid_type(&reference));
    }
}

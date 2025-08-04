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

        // Validate field modifiers
        for modifier in &field.modifiers {
            self.validate_modifier(modifier, &field.type_expr)?;
        }
        
        Ok(())
    }

    /// Validate a computed field
    fn validate_computed_field(&self, computed: &crate::ast::ComputedField) -> ParseResult<()> {
        // Create a type context for the computed field
        let context = TypeContext::new();
        
        // Add input fields to the context (they can be referenced in computed expressions)
        // Note: This would need the spec context, for now we'll do basic validation
        
        // Validate that the expression is well-formed
        let _expr_type = self.type_of_expression(&computed.expression, &context)?;
        
        Ok(())
    }

    /// Get the type of an expression in a given context
    pub fn type_of_expression(
        &self,
        expr: &crate::ast::Expression,
        context: &TypeContext,
    ) -> ParseResult<TypeExpr> {
        use crate::ast::Expression;
        
        match expr {
            Expression::StringLiteral(_) => Ok(TypeExpr::Primitive(PrimitiveType::String)),
            Expression::Number(_) => Ok(TypeExpr::Primitive(PrimitiveType::Float)),
            Expression::Boolean(_) => Ok(TypeExpr::Primitive(PrimitiveType::Bool)),
            
            Expression::Identifier(name) => {
                if let Some(var_type) = context.get_variable_type(name) {
                    Ok(var_type.clone())
                } else {
                    Err(ParseError::Type(format!("Undefined variable: {}", name)))
                }
            }
            
            Expression::FunctionCall { object, method, arguments: _ } => {
                let func_name = format!("{}.{}", object, method);
                if let Some(signature) = context.get_function(func_name.as_str()) {
                    Ok(signature.return_type.clone())
                } else {
                    // For now, assume unknown functions return strings
                    Ok(TypeExpr::Primitive(PrimitiveType::String))
                }
            }
            
            Expression::Add(left, right) | Expression::Subtract(left, right) |
            Expression::Multiply(left, right) | Expression::Divide(left, right) => {
                let left_type = self.type_of_expression(left, context)?;
                let right_type = self.type_of_expression(right, context)?;
                
                // Simple type checking: both operands should be numeric
                match (&left_type, &right_type) {
                    (TypeExpr::Primitive(PrimitiveType::Int), TypeExpr::Primitive(PrimitiveType::Int)) =>
                        Ok(TypeExpr::Primitive(PrimitiveType::Int)),
                    (TypeExpr::Primitive(PrimitiveType::Float), _) |
                    (_, TypeExpr::Primitive(PrimitiveType::Float)) =>
                        Ok(TypeExpr::Primitive(PrimitiveType::Float)),
                    _ => Err(ParseError::Type(format!(
                        "Invalid operand types for arithmetic operation: {:?} and {:?}",
                        left_type, right_type
                    )))
                }
            }
            
            Expression::Equal(left, right) | Expression::NotEqual(left, right) |
            Expression::LessThan(left, right) | Expression::LessThanOrEqual(left, right) |
            Expression::GreaterThan(left, right) | Expression::GreaterThanOrEqual(left, right) => {
                // Comparison operations return boolean
                let _left_type = self.type_of_expression(left, context)?;
                let _right_type = self.type_of_expression(right, context)?;
                // TODO: Check that types are comparable
                Ok(TypeExpr::Primitive(PrimitiveType::Bool))
            }
            
            Expression::And(left, right) | Expression::Or(left, right) => {
                let left_type = self.type_of_expression(left, context)?;
                let right_type = self.type_of_expression(right, context)?;
                
                // Both operands should be boolean
                match (&left_type, &right_type) {
                    (TypeExpr::Primitive(PrimitiveType::Bool), TypeExpr::Primitive(PrimitiveType::Bool)) =>
                        Ok(TypeExpr::Primitive(PrimitiveType::Bool)),
                    _ => Err(ParseError::Type(format!(
                        "Invalid operand types for logical operation: {:?} and {:?}",
                        left_type, right_type
                    )))
                }
            }
            
            Expression::Not(operand) => {
                let operand_type = self.type_of_expression(operand, context)?;
                match operand_type {
                    TypeExpr::Primitive(PrimitiveType::Bool) => Ok(TypeExpr::Primitive(PrimitiveType::Bool)),
                    _ => Err(ParseError::Type(format!(
                        "Invalid operand type for logical NOT: {:?}", operand_type
                    )))
                }
            }
            
            Expression::StringTemplate { parts: _ } => {
                // String templates always result in strings
                Ok(TypeExpr::Primitive(PrimitiveType::String))
            }
            
            Expression::Modulo(left, right) => {
                let left_type = self.type_of_expression(left, context)?;
                let right_type = self.type_of_expression(right, context)?;
                
                // Modulo operation on numeric types
                match (&left_type, &right_type) {
                    (TypeExpr::Primitive(PrimitiveType::Int), TypeExpr::Primitive(PrimitiveType::Int)) =>
                        Ok(TypeExpr::Primitive(PrimitiveType::Int)),
                    (TypeExpr::Primitive(PrimitiveType::Float), _) |
                    (_, TypeExpr::Primitive(PrimitiveType::Float)) =>
                        Ok(TypeExpr::Primitive(PrimitiveType::Float)),
                    _ => Err(ParseError::Type(format!(
                        "Invalid operand types for modulo operation: {:?} and {:?}",
                        left_type, right_type
                    )))
                }
            }
            
            Expression::Conditional { condition, if_true, if_false } => {
                let condition_type = self.type_of_expression(condition, context)?;
                let then_type = self.type_of_expression(if_true, context)?;
                let else_type = self.type_of_expression(if_false, context)?;
                
                // Condition must be boolean
                if !matches!(condition_type, TypeExpr::Primitive(PrimitiveType::Bool)) {
                    return Err(ParseError::Type(format!(
                        "Conditional condition must be boolean, got: {:?}", condition_type
                    )));
                }
                
                // Both branches should have compatible types
                if then_type == else_type {
                    Ok(then_type)
                } else {
                    // For now, return the then_type (could be improved with type coercion)
                    Ok(then_type)
                }
            }
            
            Expression::ArrayAccess(array_expr, index_expr) => {
                let array_type = self.type_of_expression(array_expr, context)?;
                let index_type = self.type_of_expression(index_expr, context)?;
                
                // Index should be integer
                if !matches!(index_type, TypeExpr::Primitive(PrimitiveType::Int)) {
                    return Err(ParseError::Type(format!(
                        "Array index must be integer, got: {:?}", index_type
                    )));
                }
                
                // Extract element type from array type
                match array_type {
                    TypeExpr::Generic { name, args } if name == "Array" && args.len() == 1 => {
                        Ok(args[0].clone())
                    }
                    _ => Err(ParseError::Type(format!(
                        "Cannot index non-array type: {:?}", array_type
                    )))
                }
            }
            
            Expression::PropertyAccess(object_expr, _property) => {
                let _object_type = self.type_of_expression(object_expr, context)?;
                // For now, assume property access returns string (would need struct/object type info)
                Ok(TypeExpr::Primitive(PrimitiveType::String))
            }
        }
    }
    
    /// Validate a field modifier
    fn validate_modifier(&self, modifier: &crate::ast::Modifier, field_type: &TypeExpr) -> ParseResult<()> {
        use crate::ast::Modifier;
        
        match modifier {
            Modifier::Optional => {
                // Optional modifier is always valid
                Ok(())
            }
            
            Modifier::Readonly => {
                // Readonly modifier is always valid
                Ok(())
            }
            
            Modifier::Default(expr) => {
                // Validate that the default expression type matches the field type
                let context = TypeContext::new();
                let expr_type = self.type_of_expression(expr, &context)?;
                
                if self.types_compatible(&expr_type, field_type) {
                    Ok(())
                } else {
                    Err(ParseError::Type(format!(
                        "Default value type {:?} does not match field type {:?}",
                        expr_type, field_type
                    )))
                }
            }
            
            Modifier::Computed => {
                // Computed modifier is valid for computed fields
                Ok(())
            }
            
            Modifier::Secret => {
                // Secret modifier is always valid (affects runtime behavior)
                Ok(())
            }
            
            Modifier::Generate => {
                // Generate modifier is always valid (affects runtime behavior)
                Ok(())
            }
            
            Modifier::Ref(_ref_name) => {
                // Reference modifier - would need to validate the reference exists
                // For now, assume it's valid
                Ok(())
            }
        }
    }
    
    /// Check if two types are compatible (for assignment, default values, etc.)
    fn types_compatible(&self, source_type: &TypeExpr, target_type: &TypeExpr) -> bool {
        // Exact match
        if source_type == target_type {
            return true;
        }
        
        // Numeric type compatibility
        match (source_type, target_type) {
            // Int can be assigned to Float
            (TypeExpr::Primitive(PrimitiveType::Int), TypeExpr::Primitive(PrimitiveType::Float)) => true,
            // Other cases would need more sophisticated type coercion rules
            _ => false,
        }
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

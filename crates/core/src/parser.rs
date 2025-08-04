//! # SIGMOS Parser module for SIGMOS specifications
//!
//! This module provides parsing functionality to convert SIGMOS source code
//! into an Abstract Syntax Tree (AST) representation.
//!
//! # Examples
//!
//! ```rust
//! use sigmos_core::parser::SigmosParser;
//!
//! let input = r#"
//! spec "Example" v1.0 {
//!     description: "A simple example"
//! }
//! "#;
//!
//! let spec = SigmosParser::parse_spec(input).unwrap();
//! assert_eq!(spec.name, "Example");
//! ```

use crate::ast::*;
use crate::ParseError;
use crate::ParseResult;

/// SIGMOS parser with lexical analysis and recursive descent parsing
pub struct SigmosParser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Keywords
    Spec,
    Description,
    Inputs,
    Computed,
    Events,
    Constraints,
    Lifecycle,
    Extensions,
    Types,

    // Literals
    StringLiteral(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    Identifier(String),

    // Operators and punctuation
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Colon,
    Comma,
    Arrow,
    Dot,

    // Version
    Version(u32, u32, Option<u32>),

    // End of file
    Eof,
}

impl SigmosParser {
    /// Parse a complete SIGMOS specification
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sigmos_core::parser::SigmosParser;
    ///
    /// let input = r#"
    /// spec "Test" v1.0 {
    ///     description: "Test spec"
    /// }
    /// "#;
    ///
    /// let spec = SigmosParser::parse_spec(input).unwrap();
    /// assert_eq!(spec.name, "Test");
    /// ```
    pub fn parse_spec(input: &str) -> ParseResult<Spec> {
        let mut parser = Self::new(input)?;
        parser.parse_specification()
    }

    /// Create a new parser instance
    fn new(input: &str) -> ParseResult<Self> {
        let tokens = Self::tokenize(input)?;
        Ok(Self { tokens, current: 0 })
    }

    /// Tokenize the input string
    fn tokenize(input: &str) -> ParseResult<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut chars = input.char_indices().peekable();

        while let Some((i, ch)) = chars.next() {
            match ch {
                // Skip whitespace
                ' ' | '\t' | '\n' | '\r' => continue,

                // Single character tokens
                '{' => tokens.push(Token::LeftBrace),
                '}' => tokens.push(Token::RightBrace),
                '(' => tokens.push(Token::LeftParen),
                ')' => tokens.push(Token::RightParen),
                ':' => tokens.push(Token::Colon),
                ',' => tokens.push(Token::Comma),
                '.' => tokens.push(Token::Dot),

                // Arrow ->
                '-' => {
                    if let Some((_, '>')) = chars.peek() {
                        chars.next();
                        tokens.push(Token::Arrow);
                    } else {
                        return Err(ParseError::Grammar(format!("Unexpected character: {ch}")));
                    }
                }

                // String literals
                '"' => {
                    let start = i + 1;
                    let mut end = start;
                    let mut found_end = false;

                    for (j, c) in chars.by_ref() {
                        if c == '"' {
                            end = j;
                            found_end = true;
                            break;
                        }
                    }

                    if !found_end {
                        return Err(ParseError::Grammar(
                            "Unterminated string literal".to_string(),
                        ));
                    }

                    let string_content = input[start..end].to_string();
                    tokens.push(Token::StringLiteral(string_content));
                }

                // Numbers and identifiers
                _ if ch.is_ascii_digit() => {
                    let start = i;
                    let mut end = i + 1;
                    let mut is_float = false;

                    loop {
                        match chars.peek() {
                            Some((j, c)) if c.is_ascii_digit() => {
                                let j = *j;
                                chars.next();
                                end = j + 1;
                            }
                            Some((j, '.')) if !is_float => {
                                let j = *j;
                                chars.next();
                                end = j + 1;
                                is_float = true;
                            }
                            _ => break,
                        }
                    }

                    let number_str = &input[start..end];
                    if is_float {
                        if let Ok(f) = number_str.parse::<f64>() {
                            tokens.push(Token::FloatLiteral(f));
                        } else {
                            return Err(ParseError::Grammar(format!(
                                "Invalid float literal: {number_str}"
                            )));
                        }
                    } else if let Ok(i) = number_str.parse::<i64>() {
                        tokens.push(Token::IntLiteral(i));
                    } else {
                        return Err(ParseError::Grammar(format!(
                            "Invalid integer literal: {number_str}"
                        )));
                    }
                }

                // Identifiers and keywords
                _ if ch.is_ascii_alphabetic() || ch == '_' => {
                    let start = i;
                    let mut end = i + 1;

                    loop {
                        match chars.peek() {
                            Some((j, c)) if c.is_ascii_alphanumeric() || *c == '_' => {
                                let j = *j;
                                chars.next();
                                end = j + 1;
                            }
                            _ => break,
                        }
                    }

                    let identifier = &input[start..end];

                    // Check for version pattern (v1.0, v1.2.3)
                    if identifier.starts_with('v') && identifier.len() > 1 {
                        // Look ahead to see if this is followed by a version pattern
                        let mut version_str = identifier[1..].to_string();

                        // Check if next token is a dot followed by a number
                        if let Some((_, '.')) = chars.peek() {
                            chars.next(); // consume the dot
                            version_str.push('.');

                            // Collect the minor version number
                            let mut found_minor = false;
                            while let Some((_, c)) = chars.peek() {
                                if c.is_ascii_digit() {
                                    version_str.push(*c);
                                    chars.next();
                                    found_minor = true;
                                } else {
                                    break;
                                }
                            }

                            // Check for patch version
                            if let Some((_, '.')) = chars.peek() {
                                chars.next(); // consume the dot
                                version_str.push('.');

                                while let Some((_, c)) = chars.peek() {
                                    if c.is_ascii_digit() {
                                        version_str.push(*c);
                                        chars.next();
                                    } else {
                                        break;
                                    }
                                }
                            }

                            if found_minor {
                                if let Ok(version) = Self::parse_version(&version_str) {
                                    tokens.push(Token::Version(version.0, version.1, version.2));
                                    continue;
                                }
                            }
                        }
                    }

                    // Check for keywords
                    let token = match identifier {
                        "spec" => Token::Spec,
                        "description" => Token::Description,
                        "inputs" => Token::Inputs,
                        "computed" => Token::Computed,
                        "events" => Token::Events,
                        "constraints" => Token::Constraints,
                        "lifecycle" => Token::Lifecycle,
                        "extensions" => Token::Extensions,
                        "types" => Token::Types,
                        _ => Token::Identifier(identifier.to_string()),
                    };

                    tokens.push(token);
                }

                _ => return Err(ParseError::Grammar(format!("Unexpected character: {ch}"))),
            }
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    /// Parse version string like "1.0" or "1.2.3"
    fn parse_version(
        version_str: &str,
    ) -> Result<(u32, u32, Option<u32>), Box<dyn std::error::Error>> {
        let parts: Vec<&str> = version_str.split('.').collect();

        if parts.len() < 2 || parts.len() > 3 {
            return Err("Invalid version format".into());
        }

        let major = parts[0].parse::<u32>()?;
        let minor = parts[1].parse::<u32>()?;
        let patch = if parts.len() == 3 {
            Some(parts[2].parse::<u32>()?)
        } else {
            None
        };

        Ok((major, minor, patch))
    }

    /// Parse the complete specification
    fn parse_specification(&mut self) -> ParseResult<Spec> {
        self.expect_token(Token::Spec)?;

        let name = match self.advance() {
            Token::StringLiteral(s) => s,
            _ => {
                return Err(ParseError::Grammar(
                    "Expected spec name as string literal".to_string(),
                ))
            }
        };

        let version = match self.advance() {
            Token::Version(major, minor, patch) => Version {
                major,
                minor,
                patch,
            },
            _ => {
                return Err(ParseError::Grammar(
                    "Expected version (e.g., v1.0)".to_string(),
                ))
            }
        };

        self.expect_token(Token::LeftBrace)?;

        let mut spec = Spec {
            name,
            version,
            description: None,
            inputs: Vec::new(),
            computed: Vec::new(),
            events: Vec::new(),
            constraints: Vec::new(),
            lifecycle: Vec::new(),
            extensions: Vec::new(),
            types: Vec::new(),
        };

        // Parse spec body
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.peek() {
                Token::Description => {
                    self.advance();
                    self.expect_token(Token::Colon)?;
                    if let Token::StringLiteral(desc) = self.advance() {
                        spec.description = Some(desc);
                    } else {
                        return Err(ParseError::Grammar(
                            "Expected string literal for description".to_string(),
                        ));
                    }
                }
                Token::Inputs => {
                    self.advance();
                    self.expect_token(Token::Colon)?;
                    spec.inputs = self.parse_field_list()?;
                }
                Token::Computed => {
                    self.advance();
                    self.expect_token(Token::Colon)?;
                    spec.computed = self.parse_computed_fields()?;
                }
                _ => {
                    // Skip unknown sections for now
                    self.advance();
                }
            }
        }

        self.expect_token(Token::RightBrace)?;
        Ok(spec)
    }

    /// Parse a list of field definitions
    fn parse_field_list(&mut self) -> ParseResult<Vec<FieldDef>> {
        let mut fields = Vec::new();

        while let Token::Identifier(name) = self.peek() {
            let field_name = name.clone();
            self.advance();
            self.expect_token(Token::Colon)?;

            let type_expr = self.parse_type_expr()?;
            let modifiers = Vec::new(); // TODO: Parse modifiers

            fields.push(FieldDef {
                name: field_name,
                type_expr,
                modifiers,
            });

            // Break if we don't see another identifier
            if !matches!(self.peek(), Token::Identifier(_)) {
                break;
            }
        }

        Ok(fields)
    }

    /// Parse computed field definitions
    fn parse_computed_fields(&mut self) -> ParseResult<Vec<ComputedField>> {
        let mut fields = Vec::new();

        while let Token::Identifier(name) = self.peek() {
            let field_name = name.clone();
            self.advance();
            self.expect_token(Token::Colon)?;
            self.expect_token(Token::Arrow)?;

            let expression = self.parse_expression()?;

            fields.push(ComputedField {
                name: field_name,
                expression,
            });

            // Break if we don't see another identifier
            if !matches!(self.peek(), Token::Identifier(_)) {
                break;
            }
        }

        Ok(fields)
    }

    /// Parse type expressions
    fn parse_type_expr(&mut self) -> ParseResult<TypeExpr> {
        match self.advance() {
            Token::Identifier(type_name) => match type_name.as_str() {
                "string" => Ok(TypeExpr::Primitive(PrimitiveType::String)),
                "int" => Ok(TypeExpr::Primitive(PrimitiveType::Int)),
                "float" => Ok(TypeExpr::Primitive(PrimitiveType::Float)),
                "bool" => Ok(TypeExpr::Primitive(PrimitiveType::Bool)),
                _ => Ok(TypeExpr::Reference(type_name)),
            },
            _ => Err(ParseError::Grammar("Expected type name".to_string())),
        }
    }

    /// Parse expressions
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        match self.advance() {
            Token::StringLiteral(s) => Ok(Expression::StringLiteral(s)),
            Token::IntLiteral(i) => Ok(Expression::Number(i as f64)),
            Token::FloatLiteral(f) => Ok(Expression::Number(f)),
            Token::Identifier(id) => Ok(Expression::Identifier(id)),
            _ => Err(ParseError::Grammar("Expected expression".to_string())),
        }
    }

    /// Helper methods for token management
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn check(&self, token_type: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(self.peek()) == std::mem::discriminant(token_type)
        }
    }

    fn expect_token(&mut self, expected: Token) -> ParseResult<()> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::Grammar(format!(
                "Expected {:?}, found {:?}",
                expected,
                self.peek()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_spec() {
        let input = r#"
        spec "Test" v1.0 {
            description: "A test specification"
        }
        "#;

        let result = SigmosParser::parse_spec(input);
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.name, "Test");
        assert_eq!(spec.version.major, 1);
        assert_eq!(spec.version.minor, 0);
        assert_eq!(spec.description, Some("A test specification".to_string()));
    }

    #[test]
    fn test_parse_spec_with_inputs() {
        let input = r#"
        spec "Agent" v1.0 {
            description: "An AI agent"
            inputs:
                name: string
                age: int
        }
        "#;

        let result = SigmosParser::parse_spec(input);
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.name, "Agent");
        assert_eq!(spec.inputs.len(), 2);
        assert_eq!(spec.inputs[0].name, "name");
        assert_eq!(spec.inputs[1].name, "age");
    }

    #[test]
    fn test_parse_spec_with_computed_fields() {
        let input = r#"
        spec "Example" v1.0 {
            computed:
                greeting: -> "Hello World"
                count: -> 42
        }
        "#;

        let result = SigmosParser::parse_spec(input);
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.computed.len(), 2);
        assert_eq!(spec.computed[0].name, "greeting");
        assert_eq!(spec.computed[1].name, "count");
    }
}

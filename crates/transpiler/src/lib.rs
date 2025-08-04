//! # SIGMOS Transpiler
//!
//! Transpiler for converting SIGMOS specifications to various output formats.
//!
//! Supports exporting SIGMOS AST to:
//! - JSON
//! - YAML
//! - TOML
//!
//! # Examples
//!
//! ```rust
//! use sigmos_transpiler::Transpiler;
//! use sigmos_core::ast::*;
//!
//! let transpiler = Transpiler::new();
//! let spec = Spec {
//!     name: "Test".to_string(),
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
//!
//! let json = transpiler.to_json(&spec).unwrap();
//! ```


use sigmos_core::ast::Spec;
use thiserror::Error;

/// Transpiler errors
#[derive(Error, Debug)]
pub enum TranspilerError {
    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML serialization failed: {0}")]
    Yaml(String),
    #[error("TOML serialization failed: {0}")]
    Toml(String),
}

/// Result type for transpiler operations
pub type TranspilerResult<T> = Result<T, TranspilerError>;

/// SIGMOS transpiler for converting specs to various formats
///
/// # Examples
///
/// ```rust
/// use sigmos_transpiler::Transpiler;
///
/// let transpiler = Transpiler::new();
/// ```
#[derive(Debug, Default)]
pub struct Transpiler;

impl Transpiler {
    /// Create a new transpiler instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sigmos_transpiler::Transpiler;
    ///
    /// let transpiler = Transpiler::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Convert a SIGMOS specification to JSON
    ///
    /// # Arguments
    ///
    /// * `spec` - The specification to convert
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sigmos_transpiler::Transpiler;
    /// use sigmos_core::ast::*;
    ///
    /// let transpiler = Transpiler::new();
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
    /// let json = transpiler.to_json(&spec).unwrap();
    /// assert!(json.contains("Test"));
    /// ```
    pub fn to_json(&self, spec: &Spec) -> TranspilerResult<String> {
        let json = serde_json::to_string_pretty(spec)?;
        Ok(json)
    }

    /// Convert a SIGMOS specification to YAML
    ///
    /// # Arguments
    ///
    /// * `spec` - The specification to convert
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sigmos_transpiler::Transpiler;
    /// use sigmos_core::ast::*;
    ///
    /// let transpiler = Transpiler::new();
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
    /// let yaml = transpiler.to_yaml(&spec).unwrap();
    /// assert!(yaml.contains("name: Test"));
    /// ```
    pub fn to_yaml(&self, spec: &Spec) -> TranspilerResult<String> {
        serde_yaml::to_string(spec)
            .map_err(|e| TranspilerError::Yaml(format!("YAML serialization failed: {e}")))
    }

    /// Convert a SIGMOS specification to TOML
    ///
    /// # Arguments
    ///
    /// * `spec` - The specification to convert
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sigmos_transpiler::Transpiler;
    /// use sigmos_core::ast::*;
    ///
    /// let transpiler = Transpiler::new();
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
    /// let toml_str = transpiler.to_toml(&spec).unwrap();
    /// assert!(toml_str.contains("name = \"Test\""));
    /// ```
    pub fn to_toml(&self, spec: &Spec) -> TranspilerResult<String> {
        toml::to_string(spec)
            .map_err(|e| TranspilerError::Toml(format!("TOML serialization failed: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sigmos_core::ast::*;

    fn create_test_spec() -> Spec {
        Spec {
            name: "TestSpec".to_string(),
            version: Version {
                major: 1,
                minor: 0,
                patch: None,
            },
            description: Some("A test specification".to_string()),
            inputs: vec![],
            computed: vec![],
            events: vec![],
            constraints: vec![],
            lifecycle: vec![],
            extensions: vec![],
            types: vec![],
        }
    }

    #[test]
    fn test_transpiler_creation() {
        let transpiler = Transpiler::new();
        assert!(format!("{:?}", transpiler).contains("Transpiler"));
    }

    #[test]
    fn test_to_json() {
        let transpiler = Transpiler::new();
        let spec = create_test_spec();

        let json = transpiler.to_json(&spec).unwrap();
        assert!(json.contains("TestSpec"));
        assert!(json.contains("A test specification"));
    }

    #[test]
    fn test_to_yaml() {
        let transpiler = Transpiler::new();
        let spec = create_test_spec();

        let yaml = transpiler.to_yaml(&spec).unwrap();
        assert!(yaml.contains("name: TestSpec"));
        assert!(yaml.contains("major: 1"));
        assert!(yaml.contains("minor: 0"));
    }

    #[test]
    fn test_to_toml() {
        let transpiler = Transpiler::new();
        let spec = create_test_spec();

        let toml = transpiler.to_toml(&spec).unwrap();
        assert!(toml.contains("name = \"TestSpec\""));
        assert!(toml.contains("major = 1"));
        assert!(toml.contains("minor = 0"));
    }
}

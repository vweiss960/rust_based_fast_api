//! Error types for poem_auth.
//!
//! This module defines all error types used throughout the crate.
//! Errors implement `thiserror::Error` for ergonomic error handling.

use thiserror::Error;

/// Errors that can occur during authentication.
///
/// This is the primary error type returned by auth operations.
#[derive(Debug, Error)]
pub enum AuthError {
    /// User provided invalid credentials.
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Requested user was not found.
    #[error("User not found")]
    UserNotFound,

    /// User account is disabled.
    #[error("User is disabled")]
    UserDisabled,

    /// LDAP connection or operation failed.
    #[error("LDAP error: {0}")]
    LdapError(String),

    /// Database operation failed.
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Configuration is invalid or incomplete.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// JWT operation failed (creation, validation, etc.).
    #[error("JWT error: {0}")]
    JwtError(String),

    /// Generic authentication failure.
    #[error("Authentication failed: {0}")]
    Other(String),

    /// Provider was not found or not available.
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    /// Token is invalid or expired.
    #[error("Invalid token")]
    InvalidToken,

    /// Token has expired.
    #[error("Token expired")]
    TokenExpired,

    /// Master authentication failed.
    #[error("Master authentication failed")]
    MasterAuthFailed,

    /// Rate limit exceeded.
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Secrets operation failed.
    #[error("Secrets error: {0}")]
    SecretsError(String),

    /// Password validation failed.
    #[error("Password validation failed: {0}")]
    PasswordValidationError(String),
}

impl AuthError {
    /// Create an LDAP error from a string.
    pub fn ldap<S: Into<String>>(msg: S) -> Self {
        AuthError::LdapError(msg.into())
    }

    /// Create a database error from a string.
    pub fn database<S: Into<String>>(msg: S) -> Self {
        AuthError::DatabaseError(msg.into())
    }

    /// Create a configuration error from a string.
    pub fn config<S: Into<String>>(msg: S) -> Self {
        AuthError::ConfigError(msg.into())
    }

    /// Create a JWT error from a string.
    pub fn jwt<S: Into<String>>(msg: S) -> Self {
        AuthError::JwtError(msg.into())
    }

    /// Create a generic authentication error from a string.
    pub fn other<S: Into<String>>(msg: S) -> Self {
        AuthError::Other(msg.into())
    }

    /// Check if this is an invalid credentials error (for login attempts).
    pub fn is_invalid_credentials(&self) -> bool {
        matches!(self, AuthError::InvalidCredentials)
    }

    /// Check if this is a user not found error.
    pub fn is_user_not_found(&self) -> bool {
        matches!(self, AuthError::UserNotFound)
    }

    /// Check if this is a token-related error.
    pub fn is_token_error(&self) -> bool {
        matches!(
            self,
            AuthError::InvalidToken | AuthError::TokenExpired | AuthError::JwtError(_)
        )
    }
}

/// Errors that can occur during configuration loading.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// TOML parsing error.
    #[error("Failed to parse TOML: {0}")]
    ParseError(String),

    /// Missing required configuration field.
    #[error("Missing configuration: {0}")]
    Missing(String),

    /// Configuration file not found.
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    /// File I/O error.
    #[error("I/O error: {0}")]
    IoError(String),

    /// Configuration validation failed.
    #[error("Configuration validation failed: {0}")]
    ValidationError(String),
}

impl ConfigError {
    /// Create a parse error.
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        ConfigError::ParseError(msg.into())
    }

    /// Create a missing field error.
    pub fn missing<S: Into<String>>(field: S) -> Self {
        ConfigError::Missing(field.into())
    }

    /// Create a file not found error.
    pub fn file_not_found<S: Into<String>>(path: S) -> Self {
        ConfigError::FileNotFound(path.into())
    }

    /// Create a validation error.
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        ConfigError::ValidationError(msg.into())
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err.to_string())
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}

/// Errors that can occur during secrets management.
#[derive(Debug, Error)]
pub enum SecretsError {
    /// Keyring operation failed.
    #[error("Keyring error: {0}")]
    KeyringError(String),

    /// Secret not found.
    #[error("Secret not found: {0}")]
    NotFound(String),

    /// Failed to get environment variable.
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),

    /// Invalid secret format.
    #[error("Invalid secret format: {0}")]
    InvalidFormat(String),
}

impl SecretsError {
    /// Create a keyring error.
    pub fn keyring<S: Into<String>>(msg: S) -> Self {
        SecretsError::KeyringError(msg.into())
    }

    /// Create a not found error.
    pub fn not_found<S: Into<String>>(name: S) -> Self {
        SecretsError::NotFound(name.into())
    }

    /// Create an environment variable not found error.
    pub fn env_var_not_found<S: Into<String>>(name: S) -> Self {
        SecretsError::EnvVarNotFound(name.into())
    }
}

impl From<SecretsError> for AuthError {
    fn from(err: SecretsError) -> Self {
        AuthError::SecretsError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_construction() {
        let err = AuthError::InvalidCredentials;
        assert!(err.is_invalid_credentials());
        assert!(!err.is_user_not_found());
    }

    #[test]
    fn test_auth_error_from_string() {
        let err = AuthError::ldap("Connection failed");
        assert!(matches!(err, AuthError::LdapError(_)));
    }

    #[test]
    fn test_token_error_detection() {
        assert!(AuthError::InvalidToken.is_token_error());
        assert!(AuthError::TokenExpired.is_token_error());
        assert!(!AuthError::InvalidCredentials.is_token_error());
    }

    #[test]
    fn test_config_error_creation() {
        let err = ConfigError::missing("database.path");
        assert!(matches!(err, ConfigError::Missing(_)));
    }
}

/// Configuration module for loading auth settings from TOML files
///
/// Supports loading authentication configuration (database, JWT, users) from
/// TOML files with environment variable overrides.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Complete authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Database configuration
    pub database: DatabaseConfig,
    /// JWT token configuration
    pub jwt: JwtConfig,
    /// List of users to create on startup
    pub users: Vec<UserConfig>,
    /// Optional server configuration (host, port)
    #[serde(default)]
    pub server: Option<ServerConfig>,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to SQLite database file
    pub path: String,
    /// Automatically create database if it doesn't exist (default: true)
    #[serde(default = "default_auto_create")]
    pub auto_create: bool,
}

/// JWT token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Secret key for signing tokens (must be at least 16 characters)
    pub secret: String,
    /// Token expiration time in hours (default: 24)
    #[serde(default = "default_expiration_hours")]
    pub expiration_hours: u32,
}

/// User configuration for creation on startup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// Username (unique identifier)
    pub username: String,
    /// Password (will be hashed with Argon2)
    pub password: String,
    /// List of groups/roles for this user (default: empty)
    #[serde(default)]
    pub groups: Vec<String>,
    /// Whether user is enabled (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// Server configuration (optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to (e.g., "0.0.0.0")
    pub host: String,
    /// Port to bind to (e.g., 3000)
    pub port: u16,
}

fn default_auto_create() -> bool {
    true
}

fn default_expiration_hours() -> u32 {
    24
}

fn default_enabled() -> bool {
    true
}

impl AuthConfig {
    /// Load configuration from TOML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or TOML is invalid
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = AuthConfig::from_file("auth.toml")?;
    /// config.validate()?;
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load from environment variable or fallback to file
    ///
    /// Tries to load from `AUTH_CONFIG` environment variable first,
    /// then falls back to reading from a file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to fallback TOML configuration file
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = AuthConfig::from_env_or_file("auth.toml")?;
    /// ```
    pub fn from_env_or_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(config_str) = std::env::var("AUTH_CONFIG") {
            let config = toml::from_str(&config_str)?;
            Ok(config)
        } else {
            Self::from_file(file_path)
        }
    }

    /// Validate configuration for correctness
    ///
    /// Checks:
    /// - JWT secret is at least 16 characters
    /// - Database path is not empty
    ///
    /// # Errors
    ///
    /// Returns descriptive error message if validation fails
    pub fn validate(&self) -> Result<(), String> {
        if self.jwt.secret.len() < 16 {
            return Err("JWT secret must be at least 16 characters".to_string());
        }
        if self.database.path.is_empty() {
            return Err("Database path cannot be empty".to_string());
        }
        Ok(())
    }

    /// Get server configuration with defaults
    pub fn server_config(&self) -> (String, u16) {
        match &self.server {
            Some(cfg) => (cfg.host.clone(), cfg.port),
            None => ("0.0.0.0".to_string(), 3000),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        assert_eq!(default_auto_create(), true);
        assert_eq!(default_expiration_hours(), 24);
        assert_eq!(default_enabled(), true);
    }

    #[test]
    fn test_validate_short_secret() {
        let config = AuthConfig {
            database: DatabaseConfig {
                path: "test.db".to_string(),
                auto_create: true,
            },
            jwt: JwtConfig {
                secret: "short".to_string(),
                expiration_hours: 24,
            },
            users: vec![],
            server: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_path() {
        let config = AuthConfig {
            database: DatabaseConfig {
                path: String::new(),
                auto_create: true,
            },
            jwt: JwtConfig {
                secret: "my-super-secret-key".to_string(),
                expiration_hours: 24,
            },
            users: vec![],
            server: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_valid_config() {
        let config = AuthConfig {
            database: DatabaseConfig {
                path: "test.db".to_string(),
                auto_create: true,
            },
            jwt: JwtConfig {
                secret: "my-super-secret-key".to_string(),
                expiration_hours: 24,
            },
            users: vec![],
            server: None,
        };

        assert!(config.validate().is_ok());
    }
}

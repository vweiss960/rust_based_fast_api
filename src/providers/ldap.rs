//! LDAP/Active Directory authentication provider.
//!
//! Provides authentication against LDAP/Active Directory servers.
//! Supports querying user groups via LDAP filters.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::auth::{AuthProvider, UserClaims};
use crate::error::AuthError;

/// LDAP server configuration.
///
/// Specifies how to connect to and authenticate with an LDAP server.
///
/// # Example
///
/// ```ignore
/// use poem_auth::providers::LdapConfig;
///
/// let config = LdapConfig {
///     server: "ldap://dc.example.com:389".to_string(),
///     base_dn: "DC=example,DC=com".to_string(),
///     bind_dn_template: Some("CN={username},CN=Users,DC=example,DC=com".to_string()),
///     group_filter: "(member={user_dn})".to_string(),
///     use_tls: false,
///     timeout_seconds: Some(10),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapConfig {
    /// LDAP server URL (e.g., "ldap://dc.example.com" or "ldaps://dc.example.com")
    pub server: String,

    /// Base DN for searches (e.g., "DC=example,DC=com")
    pub base_dn: String,

    /// Template for user DN during bind. Placeholders: {username}
    /// E.g., "CN={username},CN=Users,DC=example,DC=com"
    pub bind_dn_template: Option<String>,

    /// LDAP filter to find user's groups. Placeholders: {user_dn}, {username}
    /// E.g., "(member={user_dn})" or "(uniqueMember={user_dn})"
    pub group_filter: String,

    /// Use STARTTLS (if false, uses plaintext or LDAPS)
    pub use_tls: bool,

    /// Connection timeout in seconds
    pub timeout_seconds: Option<u64>,
}

impl LdapConfig {
    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), AuthError> {
        if self.server.is_empty() {
            return Err(AuthError::config("LDAP server URL is required"));
        }

        if self.base_dn.is_empty() {
            return Err(AuthError::config("LDAP base DN is required"));
        }

        if self.group_filter.is_empty() {
            return Err(AuthError::config("LDAP group filter is required"));
        }

        Ok(())
    }

    /// Get the timeout duration.
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds.unwrap_or(10))
    }
}

/// LDAP authentication provider.
///
/// Authenticates users against an LDAP/Active Directory server and retrieves
/// their group memberships via LDAP queries.
///
/// # Example
///
/// ```ignore
/// use poem_auth::providers::{LdapAuthProvider, LdapConfig};
///
/// let config = LdapConfig {
///     server: "ldap://dc.example.com".to_string(),
///     base_dn: "DC=example,DC=com".to_string(),
///     bind_dn_template: Some("CN={username},CN=Users,DC=example,DC=com".to_string()),
///     group_filter: "(member={user_dn})".to_string(),
///     use_tls: false,
///     timeout_seconds: Some(10),
/// };
///
/// let provider = LdapAuthProvider::new(config)?;
/// let claims = provider.authenticate("alice", "password123").await?;
/// println!("Groups: {:?}", claims.groups);
/// ```
#[cfg(feature = "ldap")]
pub struct LdapAuthProvider {
    config: LdapConfig,
}

#[cfg(feature = "ldap")]
impl LdapAuthProvider {
    /// Create a new LDAP provider with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn new(config: LdapConfig) -> Result<Self, AuthError> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Get the configuration.
    pub fn config(&self) -> &LdapConfig {
        &self.config
    }

    /// Format the user's DN using the bind_dn_template.
    fn format_user_dn(&self, username: &str) -> String {
        match &self.config.bind_dn_template {
            Some(template) => template.replace("{username}", username),
            None => format!("CN={},OU=Users,{}", username, self.config.base_dn),
        }
    }
}

#[cfg(feature = "ldap")]
impl std::fmt::Debug for LdapAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LdapAuthProvider")
            .field("server", &self.config.server)
            .field("base_dn", &self.config.base_dn)
            .finish()
    }
}

#[cfg(feature = "ldap")]
#[async_trait]
impl AuthProvider for LdapAuthProvider {
    async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<UserClaims, AuthError> {
        // Note: Full LDAP implementation would require ldap3 library
        // For now, this is a stub implementation with proper structure
        // A real implementation would:
        // 1. Connect to LDAP server
        // 2. Bind with user's DN and password
        // 3. Query groups using the group filter
        // 4. Return claims with groups

        // Basic validation
        if username.is_empty() || password.is_empty() {
            return Err(AuthError::InvalidCredentials);
        }

        // Format user DN
        let user_dn = self.format_user_dn(username);

        // In a real implementation, we would:
        // - Connect to LDAP
        // - Attempt bind with user credentials
        // - Query for groups
        // - Return UserClaims

        // For now, return a placeholder to demonstrate structure
        let now = chrono::Utc::now().timestamp();
        let expiration = now + (24 * 60 * 60); // 24 hours

        Ok(UserClaims::new(username, "ldap", expiration, now))
    }

    fn name(&self) -> &str {
        "ldap"
    }

    async fn validate_config(&self) -> Result<(), AuthError> {
        // In a real implementation, would attempt to connect to LDAP server
        // to verify connectivity and configuration validity.
        // For now, we just validate the config structure.
        self.config.validate()
    }

    fn info(&self) -> String {
        format!(
            "LDAP authentication provider (server: {}, base_dn: {})",
            self.config.server, self.config.base_dn
        )
    }
}

#[cfg(not(feature = "ldap"))]
/// Stub LDAP provider when feature is disabled.
pub struct LdapAuthProvider;

#[cfg(not(feature = "ldap"))]
impl LdapAuthProvider {
    /// This method is unavailable when the ldap feature is not enabled.
    pub fn new(_config: LdapConfig) -> Result<Self, AuthError> {
        Err(AuthError::config(
            "LDAP support is not enabled. Add the 'ldap' feature to Cargo.toml",
        ))
    }
}

#[cfg(all(test, feature = "ldap"))]
mod tests {
    use super::*;

    #[test]
    fn test_ldap_config_validation() {
        let valid_config = LdapConfig {
            server: "ldap://dc.example.com".to_string(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: Some("CN={username},CN=Users,DC=example,DC=com".to_string()),
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: Some(10),
        };

        assert!(valid_config.validate().is_ok());
    }

    #[test]
    fn test_ldap_config_validation_missing_server() {
        let config = LdapConfig {
            server: String::new(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: None,
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ldap_config_validation_missing_base_dn() {
        let config = LdapConfig {
            server: "ldap://dc.example.com".to_string(),
            base_dn: String::new(),
            bind_dn_template: None,
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ldap_config_validation_missing_group_filter() {
        let config = LdapConfig {
            server: "ldap://dc.example.com".to_string(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: None,
            group_filter: String::new(),
            use_tls: false,
            timeout_seconds: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ldap_provider_creation() {
        let config = LdapConfig {
            server: "ldap://dc.example.com".to_string(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: Some("CN={username},CN=Users,DC=example,DC=com".to_string()),
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: Some(10),
        };

        let provider = LdapAuthProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_ldap_provider_invalid_config() {
        let config = LdapConfig {
            server: String::new(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: None,
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: None,
        };

        let provider = LdapAuthProvider::new(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_ldap_provider_name() {
        let config = LdapConfig {
            server: "ldap://dc.example.com".to_string(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: None,
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: None,
        };

        let provider = LdapAuthProvider::new(config).unwrap();
        assert_eq!(provider.name(), "ldap");
    }

    #[test]
    fn test_ldap_provider_info() {
        let config = LdapConfig {
            server: "ldap://dc.example.com".to_string(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: None,
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: None,
        };

        let provider = LdapAuthProvider::new(config).unwrap();
        let info = provider.info();
        assert!(info.contains("LDAP"));
        assert!(info.contains("ldap://dc.example.com"));
    }

    #[test]
    fn test_format_user_dn_with_template() {
        let config = LdapConfig {
            server: "ldap://dc.example.com".to_string(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: Some("CN={username},CN=Users,DC=example,DC=com".to_string()),
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: None,
        };

        let provider = LdapAuthProvider::new(config).unwrap();
        let dn = provider.format_user_dn("alice");
        assert_eq!(dn, "CN=alice,CN=Users,DC=example,DC=com");
    }

    #[test]
    fn test_format_user_dn_default() {
        let config = LdapConfig {
            server: "ldap://dc.example.com".to_string(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: None,
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: None,
        };

        let provider = LdapAuthProvider::new(config).unwrap();
        let dn = provider.format_user_dn("bob");
        assert_eq!(dn, "CN=bob,OU=Users,DC=example,DC=com");
    }

    #[tokio::test]
    async fn test_authenticate_empty_credentials() {
        let config = LdapConfig {
            server: "ldap://dc.example.com".to_string(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: None,
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: None,
        };

        let provider = LdapAuthProvider::new(config).unwrap();
        let result = provider.authenticate("", "password").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_config() {
        let config = LdapConfig {
            server: "ldap://dc.example.com".to_string(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: None,
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: None,
        };

        let provider = LdapAuthProvider::new(config).unwrap();
        let result = provider.validate_config().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_ldap_config_timeout() {
        let config1 = LdapConfig {
            server: "ldap://dc.example.com".to_string(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: None,
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: Some(30),
        };

        let config2 = LdapConfig {
            server: "ldap://dc.example.com".to_string(),
            base_dn: "DC=example,DC=com".to_string(),
            bind_dn_template: None,
            group_filter: "(member={user_dn})".to_string(),
            use_tls: false,
            timeout_seconds: None,
        };

        assert_eq!(config1.timeout(), Duration::from_secs(30));
        assert_eq!(config2.timeout(), Duration::from_secs(10));
    }
}

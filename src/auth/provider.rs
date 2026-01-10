//! Core authentication provider trait.
//!
//! The `AuthProvider` trait is the main extension point for poem_auth.
//! Implement this trait to add support for custom authentication methods.

use async_trait::async_trait;
use std::fmt::Debug;

use crate::error::AuthError;
use super::claims::UserClaims;

/// Core trait for authentication implementations.
///
/// Implement this trait to add custom authentication methods such as:
/// - OAuth2
/// - SAML
/// - Custom LDAP variants
/// - API key authentication
/// - Multi-factor authentication
///
/// # Example
///
/// ```ignore
/// use poem_auth::AuthProvider;
/// use poem_auth::UserClaims;
/// use poem_auth::AuthError;
/// use async_trait::async_trait;
///
/// struct MyCustomProvider {
///     // Your configuration
/// }
///
/// #[async_trait]
/// impl AuthProvider for MyCustomProvider {
///     async fn authenticate(
///         &self,
///         username: &str,
///         password: &str,
///     ) -> Result<UserClaims, AuthError> {
///         // Your authentication logic
///         todo!()
///     }
///
///     fn name(&self) -> &str {
///         "custom"
///     }
/// }
/// ```
#[async_trait]
pub trait AuthProvider: Send + Sync + Debug {
    /// Authenticate a user and return their claims.
    ///
    /// This method should verify the user's credentials and return a `UserClaims`
    /// struct containing the authenticated user's information, including their
    /// username, groups, and any other relevant metadata.
    ///
    /// # Arguments
    ///
    /// * `username` - The user's username or identifier
    /// * `password` - The user's password or credential
    ///
    /// # Returns
    ///
    /// * `Ok(UserClaims)` if authentication succeeds
    /// * `Err(AuthError)` if authentication fails
    ///
    /// # Errors
    ///
    /// Common errors include:
    /// - `AuthError::InvalidCredentials` - Wrong password
    /// - `AuthError::UserNotFound` - User doesn't exist
    /// - `AuthError::UserDisabled` - User account is disabled
    /// - `AuthError::LdapError` - LDAP connection failed
    /// - `AuthError::DatabaseError` - Database operation failed
    async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<UserClaims, AuthError>;

    /// Get the provider's display name.
    ///
    /// This name is used for:
    /// - Logging and debugging
    /// - The `provider` field in JWT claims
    /// - Configuration identification
    ///
    /// Should return a short, lowercase identifier (e.g., "local", "ldap", "oauth2").
    fn name(&self) -> &str;

    /// Validate provider configuration.
    ///
    /// Called during initialization to catch configuration errors early.
    /// Override this method to validate that the provider is properly configured
    /// before any authentication attempts are made.
    ///
    /// For example, LDAP provider might validate server connectivity,
    /// LocalAuthProvider might validate database connectivity.
    ///
    /// Default implementation does nothing (returns Ok).
    async fn validate_config(&self) -> Result<(), AuthError> {
        Ok(())
    }

    /// Get human-readable information about this provider.
    ///
    /// Used for documentation and debugging purposes.
    /// Override to provide useful information about the provider's configuration.
    fn info(&self) -> String {
        format!("Provider: {}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockProvider;

    #[async_trait]
    impl AuthProvider for MockProvider {
        async fn authenticate(
            &self,
            _username: &str,
            _password: &str,
        ) -> Result<UserClaims, AuthError> {
            Ok(UserClaims {
                sub: "test".to_string(),
                groups: vec![],
                provider: "mock".to_string(),
                exp: 0,
                iat: 0,
                jti: "test-jti".to_string(),
                extra: None,
            })
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_auth_provider_trait() {
        let provider = MockProvider;
        assert_eq!(provider.name(), "mock");

        let result = provider.authenticate("user", "pass").await;
        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub, "test");
    }

    #[tokio::test]
    async fn test_validate_config_default() {
        let provider = MockProvider;
        let result = provider.validate_config().await;
        assert!(result.is_ok());
    }
}

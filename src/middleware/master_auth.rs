//! Master authentication for administrative endpoints.
//!
//! Provides authentication using a master username/password pair to protect
//! administrative operations like user creation, deletion, and configuration changes.

use crate::error::AuthError;
use crate::password;

/// Master credentials for administrative access.
///
/// Used to authenticate requests to protected administrative endpoints.
/// The master password hash is stored securely and compared using constant-time comparison.
///
/// # Example
///
/// ```ignore
/// use poem_auth::middleware::MasterCredentials;
///
/// async fn admin_handler(creds: MasterCredentials) -> impl IntoResponse {
///     // Only reached if master auth succeeded
///     format!("Admin authenticated as: {}", creds.username)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MasterCredentials {
    /// The master username (typically "admin" or "master")
    pub username: String,
}

impl MasterCredentials {
    /// Create new master credentials.
    pub fn new(username: String) -> Self {
        Self { username }
    }
}

/// Master authentication validator.
///
/// Validates that provided credentials match the stored master password hash.
/// Uses constant-time comparison to prevent timing attacks.
///
/// # Example
///
/// ```ignore
/// use poem_auth::middleware::MasterAuth;
/// use poem_auth::password::hash_password;
///
/// let master_hash = hash_password("your-master-password")?;
/// let auth = MasterAuth::new("admin", &master_hash);
///
/// // Verify credentials
/// let creds = auth.validate("admin", "your-master-password")?;
/// assert_eq!(creds.username, "admin");
/// ```
#[derive(Debug, Clone)]
pub struct MasterAuth {
    /// Master username (typically "admin")
    username: String,
    /// Argon2 hash of the master password
    password_hash: String,
}

impl MasterAuth {
    /// Create a new master auth validator.
    ///
    /// # Arguments
    ///
    /// * `username` - The master username (e.g., "admin")
    /// * `password_hash` - An Argon2 hash of the master password
    ///
    /// # Example
    ///
    /// ```ignore
    /// let master_hash = hash_password("secure-password")?;
    /// let master_auth = MasterAuth::new("admin", &master_hash);
    /// ```
    pub fn new(username: &str, password_hash: &str) -> Self {
        Self {
            username: username.to_string(),
            password_hash: password_hash.to_string(),
        }
    }

    /// Validate master credentials.
    ///
    /// Checks that the provided username and password match the master credentials.
    /// Returns `MasterCredentials` on success, or `AuthError` on failure.
    ///
    /// # Arguments
    ///
    /// * `username` - The provided username
    /// * `password` - The provided password (plaintext)
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InvalidCredentials` if username doesn't match or password is wrong.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let creds = master_auth.validate("admin", "correct-password")?;
    /// ```
    pub fn validate(&self, username: &str, password: &str) -> Result<MasterCredentials, AuthError> {
        // Check username matches
        if username != self.username {
            return Err(AuthError::InvalidCredentials);
        }

        // Verify password hash
        password::verify_password(password, &self.password_hash)?;

        Ok(MasterCredentials::new(self.username.clone()))
    }

    /// Get the master username.
    pub fn username(&self) -> &str {
        &self.username
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_credentials_creation() {
        let creds = MasterCredentials::new("admin".to_string());
        assert_eq!(creds.username, "admin");
    }

    #[test]
    fn test_master_auth_creation() {
        let hash = crate::password::hash_password("test-password").unwrap();
        let auth = MasterAuth::new("admin", &hash);
        assert_eq!(auth.username(), "admin");
    }

    #[test]
    fn test_master_auth_validate_success() {
        let test_password = "my-secure-master-password";
        let hash = crate::password::hash_password(test_password).unwrap();
        let auth = MasterAuth::new("admin", &hash);

        let result = auth.validate("admin", test_password);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().username, "admin");
    }

    #[test]
    fn test_master_auth_validate_wrong_username() {
        let test_password = "my-secure-master-password";
        let hash = crate::password::hash_password(test_password).unwrap();
        let auth = MasterAuth::new("admin", &hash);

        let result = auth.validate("wronguser", test_password);
        assert!(result.is_err());
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[test]
    fn test_master_auth_validate_wrong_password() {
        let test_password = "my-secure-master-password";
        let hash = crate::password::hash_password(test_password).unwrap();
        let auth = MasterAuth::new("admin", &hash);

        let result = auth.validate("admin", "wrong-password");
        assert!(result.is_err());
    }

    #[test]
    fn test_master_auth_validate_empty_password() {
        let test_password = "my-secure-master-password";
        let hash = crate::password::hash_password(test_password).unwrap();
        let auth = MasterAuth::new("admin", &hash);

        let result = auth.validate("admin", "");
        assert!(result.is_err());
    }

    #[test]
    fn test_master_auth_different_instances_compatible() {
        let test_password = "shared-password";
        let hash = crate::password::hash_password(test_password).unwrap();

        let auth1 = MasterAuth::new("admin", &hash);
        let auth2 = MasterAuth::new("admin", &hash);

        // Both instances should validate the same password successfully
        assert!(auth1.validate("admin", test_password).is_ok());
        assert!(auth2.validate("admin", test_password).is_ok());
    }
}

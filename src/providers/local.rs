//! Local authentication provider backed by a user database.
//!
//! This provider authenticates users by looking up their record in a `UserDatabase`
//! and verifying the password hash.

use async_trait::async_trait;
use std::sync::Arc;

use crate::auth::{AuthProvider, UserClaims};
use crate::db::UserDatabase;
use crate::error::AuthError;
use crate::password;

/// Authentication provider backed by a local user database.
///
/// Authenticates users by:
/// 1. Looking up the username in the database
/// 2. Verifying the password against the stored Argon2 hash
/// 3. Checking that the user is enabled
/// 4. Returning claims with the user's username and groups
///
/// # Example
///
/// ```ignore
/// use poem_auth::providers::LocalAuthProvider;
/// use poem_auth::db::sqlite::SqliteUserDb;
///
/// let db = SqliteUserDb::new("users.db").await?;
/// let provider = LocalAuthProvider::new(db);
///
/// let claims = provider.authenticate("alice", "password123").await?;
/// println!("Logged in as: {}", claims.sub);
/// ```
#[derive(Debug)]
pub struct LocalAuthProvider {
    db: Arc<dyn UserDatabase>,
}

impl LocalAuthProvider {
    /// Create a new local authentication provider.
    ///
    /// # Arguments
    ///
    /// * `db` - A user database implementation
    pub fn new<D: UserDatabase + 'static>(db: D) -> Self {
        Self {
            db: Arc::new(db),
        }
    }

    /// Create a new provider with an Arc-wrapped database.
    pub fn with_db(db: Arc<dyn UserDatabase>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AuthProvider for LocalAuthProvider {
    async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<UserClaims, AuthError> {
        // Get user from database
        let user = self.db.get_user(username).await?;

        // Check if user is enabled
        if !user.enabled {
            return Err(AuthError::UserDisabled);
        }

        // Verify password hash
        password::verify_password(password, &user.password_hash)?;

        // Generate claims
        let now = chrono::Utc::now().timestamp();
        let expiration = now + (24 * 60 * 60); // 24 hours default

        Ok(UserClaims::new(username, "local", expiration, now)
            .with_groups(user.groups))
    }

    fn name(&self) -> &str {
        "local"
    }

    async fn validate_config(&self) -> Result<(), AuthError> {
        // Try to list users to validate database connectivity
        self.db.list_users().await?;
        Ok(())
    }

    fn info(&self) -> String {
        "Local database authentication provider".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::sqlite::SqliteUserDb;
    use crate::password;
    use tempfile::TempDir;

    async fn test_provider() -> Result<LocalAuthProvider, AuthError> {
        let temp_dir = TempDir::new()
            .map_err(|e| AuthError::database(e.to_string()))?;
        let db_path = temp_dir.path().join("test.db");
        let path = db_path.to_str()
            .ok_or_else(|| AuthError::database("Invalid temp path".to_string()))?;

        let db = SqliteUserDb::new(path).await?;
        std::mem::forget(temp_dir);
        let provider = LocalAuthProvider::new(db);

        // Create a test user
        let password_hash = password::hash_password("test123")?;
        let user = crate::db::UserRecord::new("alice", &password_hash)
            .with_groups(vec!["admins", "users"]);
        provider.db.create_user(user).await?;

        Ok(provider)
    }

    #[tokio::test]
    async fn test_authenticate_success() {
        let provider = test_provider().await.unwrap();
        let claims = provider.authenticate("alice", "test123").await.unwrap();

        assert_eq!(claims.sub, "alice");
        assert_eq!(claims.provider, "local");
        assert_eq!(claims.groups, vec!["admins", "users"]);
        assert!(claims.iat > 0);
        assert!(claims.exp > claims.iat);
    }

    #[tokio::test]
    async fn test_authenticate_wrong_password() {
        let provider = test_provider().await.unwrap();
        assert!(provider.authenticate("alice", "wrong_password").await.is_err());
    }

    #[tokio::test]
    async fn test_authenticate_user_not_found() {
        let provider = test_provider().await.unwrap();
        assert!(provider.authenticate("nonexistent", "password").await.is_err());
    }

    #[tokio::test]
    async fn test_authenticate_disabled_user() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let path = db_path.to_str().unwrap();
        let db = SqliteUserDb::new(path).await.unwrap();
        std::mem::forget(temp_dir);

        let password_hash = password::hash_password("test123").unwrap();
        let user = crate::db::UserRecord::new("bob", &password_hash)
            .disable();
        db.create_user(user).await.unwrap();

        let provider = LocalAuthProvider::new(db);
        assert!(provider.authenticate("bob", "test123").await.is_err());
    }

    #[tokio::test]
    async fn test_name() {
        let provider = test_provider().await.unwrap();
        assert_eq!(provider.name(), "local");
    }

    #[tokio::test]
    async fn test_info() {
        let provider = test_provider().await.unwrap();
        assert!(provider.info().contains("Local database"));
    }

    #[tokio::test]
    async fn test_validate_config() {
        let provider = test_provider().await.unwrap();
        assert!(provider.validate_config().await.is_ok());
    }

    #[tokio::test]
    async fn test_claims_include_groups() {
        let provider = test_provider().await.unwrap();
        let claims = provider.authenticate("alice", "test123").await.unwrap();

        assert!(claims.has_group("admins"));
        assert!(claims.has_group("users"));
        assert!(!claims.has_group("nonexistent"));
    }

    #[tokio::test]
    async fn test_claims_expiration() {
        let provider = test_provider().await.unwrap();
        let claims = provider.authenticate("alice", "test123").await.unwrap();

        // Token should expire in approximately 24 hours
        let ttl = claims.time_to_expiry(claims.iat);
        assert!(ttl > 86000);  // A bit less than 24 hours to account for processing time
        assert!(ttl <= 86400); // Exactly 24 hours
    }
}

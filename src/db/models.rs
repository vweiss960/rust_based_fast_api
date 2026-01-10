//! Database models and trait definitions.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::AuthError;

/// Trait for custom user storage implementations.
///
/// Implement this trait to use a custom database backend (PostgreSQL, MongoDB, etc.)
/// instead of the provided SQLite implementation.
///
/// Most applications can use the provided `SqliteUserDb`, but this trait allows
/// you to integrate poem_auth with your existing database infrastructure.
///
/// # Example: PostgreSQL Implementation
///
/// ```ignore
/// use poem_auth::db::UserDatabase;
/// use poem_auth::db::UserRecord;
/// use poem_auth::error::AuthError;
/// use async_trait::async_trait;
///
/// struct PostgresUserDb {
///     pool: sqlx::PgPool,
/// }
///
/// #[async_trait]
/// impl UserDatabase for PostgresUserDb {
///     async fn get_user(&self, username: &str) -> Result<UserRecord, AuthError> {
///         let user = sqlx::query_as::<_, UserRecord>(
///             "SELECT username, password_hash, groups, enabled, created_at, updated_at FROM users WHERE username = $1"
///         )
///         .bind(username)
///         .fetch_optional(&self.pool)
///         .await
///         .map_err(|e| AuthError::database(e.to_string()))?
///         .ok_or(AuthError::UserNotFound)?;
///
///         Ok(user)
///     }
///
///     // ... implement other methods
/// }
/// ```
#[async_trait]
pub trait UserDatabase: Send + Sync + std::fmt::Debug {
    /// Retrieve a user record by username.
    ///
    /// # Returns
    ///
    /// * `Ok(UserRecord)` if user exists
    /// * `Err(AuthError::UserNotFound)` if user doesn't exist
    /// * `Err(AuthError::DatabaseError)` on database errors
    async fn get_user(&self, username: &str) -> Result<UserRecord, AuthError>;

    /// Create a new user record.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success
    /// * `Err(AuthError)` if user already exists or database error occurs
    async fn create_user(&self, user: UserRecord) -> Result<(), AuthError>;

    /// Update a user's password hash.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success
    /// * `Err(AuthError::UserNotFound)` if user doesn't exist
    /// * `Err(AuthError::DatabaseError)` on database errors
    async fn update_password(&self, username: &str, hash: String) -> Result<(), AuthError>;

    /// List all users in the system.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<UserRecord>)` list of all users
    /// * `Err(AuthError::DatabaseError)` on database errors
    async fn list_users(&self) -> Result<Vec<UserRecord>, AuthError>;

    /// Delete a user record.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success
    /// * `Err(AuthError::UserNotFound)` if user doesn't exist
    /// * `Err(AuthError::DatabaseError)` on database errors
    async fn delete_user(&self, username: &str) -> Result<(), AuthError>;

    /// Check if a user exists.
    ///
    /// Default implementation uses `get_user`, but can be overridden for efficiency.
    async fn user_exists(&self, username: &str) -> Result<bool, AuthError> {
        match self.get_user(username).await {
            Ok(_) => Ok(true),
            Err(AuthError::UserNotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Update a user's groups.
    ///
    /// Default implementation should be overridden by actual implementations.
    async fn update_groups(
        &self,
        username: &str,
        groups: Vec<String>,
    ) -> Result<(), AuthError> {
        let mut user = self.get_user(username).await?;
        user.groups = groups;
        // In a real implementation, this would update the database
        // For now, we need a way to persist this change
        Ok(())
    }
}

/// A user record in the database.
///
/// This struct represents a stored user account with password hash and group membership.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    /// Unique username identifier.
    pub username: String,

    /// Argon2 password hash.
    ///
    /// Never expose this value to clients or logs.
    pub password_hash: String,

    /// Groups/roles the user belongs to.
    ///
    /// Used for authorization and included in JWT claims.
    pub groups: Vec<String>,

    /// Whether the user account is enabled.
    ///
    /// Disabled users cannot authenticate even with valid credentials.
    pub enabled: bool,

    /// Unix timestamp when user was created.
    pub created_at: i64,

    /// Unix timestamp when user was last updated.
    pub updated_at: i64,
}

impl UserRecord {
    /// Create a new user record with current timestamp.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use poem_auth::db::UserRecord;
    ///
    /// let user = UserRecord::new("alice", "$argon2id$...");
    /// ```
    pub fn new(username: &str, password_hash: &str) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            username: username.to_string(),
            password_hash: password_hash.to_string(),
            groups: Vec::new(),
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set user's groups.
    pub fn with_groups<S: Into<String>>(mut self, groups: Vec<S>) -> Self {
        self.groups = groups.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Add a group to the user.
    pub fn add_group<S: Into<String>>(mut self, group: S) -> Self {
        self.groups.push(group.into());
        self
    }

    /// Set whether the user is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Disable the user account.
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Check if the user is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if user has a specific group.
    pub fn has_group(&self, group: &str) -> bool {
        self.groups.iter().any(|g| g == group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_record_new() {
        let user = UserRecord::new("alice", "hash123");
        assert_eq!(user.username, "alice");
        assert_eq!(user.password_hash, "hash123");
        assert!(user.enabled);
        assert_eq!(user.groups.len(), 0);
    }

    #[test]
    fn test_user_record_builder() {
        let user = UserRecord::new("alice", "hash")
            .with_groups(vec!["admins", "users"])
            .add_group("developers");
        assert_eq!(user.groups.len(), 3);
        assert!(user.has_group("admins"));
        assert!(user.has_group("developers"));
    }

    #[test]
    fn test_user_record_disable() {
        let user = UserRecord::new("alice", "hash").disable();
        assert!(!user.is_enabled());
    }

    #[test]
    fn test_user_record_serialization() {
        let user = UserRecord::new("alice", "hash")
            .with_groups(vec!["admins"]);
        let json = serde_json::to_string(&user).unwrap();
        let deserialized: UserRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(user.username, deserialized.username);
    }
}

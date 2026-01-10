//! SQLite-based user database implementation.
//!
//! This module provides a ready-to-use SQLite implementation of the `UserDatabase` trait.
//! It's included when the `sqlite` feature is enabled.

use async_trait::async_trait;

use crate::db::{UserDatabase, UserRecord};
use crate::error::AuthError;

/// SQLite-backed user database.
///
/// Provides a complete implementation of the `UserDatabase` trait using SQLite.
/// This is the default and recommended database backend for most applications.
///
/// # Example
///
/// ```ignore
/// use poem_auth::db::sqlite::SqliteUserDb;
///
/// let db = SqliteUserDb::new("data/users.db").await?;
/// let user = db.get_user("alice").await?;
/// ```
#[derive(Debug, Clone)]
pub struct SqliteUserDb {
    // Connection pool will be added in Phase 2
    // For now, this is a placeholder
}

impl SqliteUserDb {
    /// Create a new SQLite database.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SQLite database file
    ///
    /// # Example
    ///
    /// ```ignore
    /// let db = SqliteUserDb::new("data/users.db").await?;
    /// ```
    pub async fn new(_path: &str) -> Result<Self, AuthError> {
        // Implementation will be added in Phase 2
        todo!("SQLite implementation will be added in Phase 2")
    }

    /// Run database migrations.
    pub async fn migrate(&self) -> Result<(), AuthError> {
        // Implementation will be added in Phase 2
        todo!("Migrations will be added in Phase 2")
    }
}

#[async_trait]
impl UserDatabase for SqliteUserDb {
    async fn get_user(&self, _username: &str) -> Result<UserRecord, AuthError> {
        todo!("Implementation in Phase 2")
    }

    async fn create_user(&self, _user: UserRecord) -> Result<(), AuthError> {
        todo!("Implementation in Phase 2")
    }

    async fn update_password(&self, _username: &str, _hash: String) -> Result<(), AuthError> {
        todo!("Implementation in Phase 2")
    }

    async fn list_users(&self) -> Result<Vec<UserRecord>, AuthError> {
        todo!("Implementation in Phase 2")
    }

    async fn delete_user(&self, _username: &str) -> Result<(), AuthError> {
        todo!("Implementation in Phase 2")
    }
}

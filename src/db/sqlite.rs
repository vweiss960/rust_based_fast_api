//! SQLite-based user database implementation.
//!
//! This module provides a ready-to-use SQLite implementation of the `UserDatabase` trait.
//! It's included when the `sqlite` feature is enabled.

use async_trait::async_trait;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use sqlx::ConnectOptions;
use std::str::FromStr;
use std::sync::Arc;

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
    pool: Arc<SqlitePool>,
}

impl SqliteUserDb {
    /// Create a new SQLite database.
    ///
    /// Creates the database file if it doesn't exist and runs migrations.
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
    pub async fn new(path: &str) -> Result<Self, AuthError> {
        // Create connection options with minimal verbosity
        let connect_options = SqliteConnectOptions::from_str(path)
            .map_err(|e| AuthError::database(format!("Invalid database path: {}", e)))?
            .create_if_missing(true)
            .log_statements(tracing::log::LevelFilter::Debug);

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connect_options)
            .await
            .map_err(|e| AuthError::database(format!("Failed to connect to database: {}", e)))?;

        // Create database
        let db = Self {
            pool: Arc::new(pool),
        };

        // Run migrations
        db.migrate().await?;

        Ok(db)
    }

    /// Run database migrations to create schema.
    pub async fn migrate(&self) -> Result<(), AuthError> {
        // Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                groups TEXT NOT NULL DEFAULT '[]',
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AuthError::database(format!("Failed to create users table: {}", e)))?;

        // Create audit_log table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                username TEXT,
                provider TEXT NOT NULL,
                ip_address TEXT,
                details TEXT
            )
            "#,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AuthError::database(format!("Failed to create audit_log table: {}", e)))?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)")
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| AuthError::database(format!("Failed to create index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp)")
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| AuthError::database(format!("Failed to create index: {}", e)))?;

        Ok(())
    }

    /// Get the underlying connection pool for custom queries.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl UserDatabase for SqliteUserDb {
    async fn get_user(&self, username: &str) -> Result<UserRecord, AuthError> {
        let user = sqlx::query_as::<_, (String, String, String, bool, i64, i64)>(
            "SELECT username, password_hash, groups, enabled, created_at, updated_at FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AuthError::database(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        let groups: Vec<String> = serde_json::from_str(&user.2)
            .unwrap_or_default();

        Ok(UserRecord {
            username: user.0,
            password_hash: user.1,
            groups,
            enabled: user.3,
            created_at: user.4,
            updated_at: user.5,
        })
    }

    async fn create_user(&self, user: UserRecord) -> Result<(), AuthError> {
        let groups_json = serde_json::to_string(&user.groups)
            .map_err(|e| AuthError::database(format!("Failed to serialize groups: {}", e)))?;

        sqlx::query(
            "INSERT INTO users (username, password_hash, groups, enabled, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&groups_json)
        .bind(user.enabled)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                AuthError::other(format!("User '{}' already exists", user.username))
            } else {
                AuthError::database(e.to_string())
            }
        })?;

        Ok(())
    }

    async fn update_password(&self, username: &str, hash: String) -> Result<(), AuthError> {
        let now = chrono::Utc::now().timestamp();

        let result = sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE username = ?")
            .bind(&hash)
            .bind(now)
            .bind(username)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| AuthError::database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::UserNotFound);
        }

        Ok(())
    }

    async fn list_users(&self) -> Result<Vec<UserRecord>, AuthError> {
        let rows = sqlx::query_as::<_, (String, String, String, bool, i64, i64)>(
            "SELECT username, password_hash, groups, enabled, created_at, updated_at FROM users ORDER BY username"
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AuthError::database(e.to_string()))?;

        let users = rows.into_iter().map(|row| {
            let groups: Vec<String> = serde_json::from_str(&row.2)
                .unwrap_or_default();

            UserRecord {
                username: row.0,
                password_hash: row.1,
                groups,
                enabled: row.3,
                created_at: row.4,
                updated_at: row.5,
            }
        }).collect();

        Ok(users)
    }

    async fn delete_user(&self, username: &str) -> Result<(), AuthError> {
        let result = sqlx::query("DELETE FROM users WHERE username = ?")
            .bind(username)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| AuthError::database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::UserNotFound);
        }

        Ok(())
    }

    async fn update_groups(&self, username: &str, groups: Vec<String>) -> Result<(), AuthError> {
        let now = chrono::Utc::now().timestamp();
        let groups_json = serde_json::to_string(&groups)
            .map_err(|e| AuthError::database(format!("Failed to serialize groups: {}", e)))?;

        let result = sqlx::query("UPDATE users SET groups = ?, updated_at = ? WHERE username = ?")
            .bind(&groups_json)
            .bind(now)
            .bind(username)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| AuthError::database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::UserNotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_db() -> Result<SqliteUserDb, AuthError> {
        let temp_dir = TempDir::new()
            .map_err(|e| AuthError::database(e.to_string()))?;
        let db_path = temp_dir.path().join("test.db");
        let path = db_path.to_str()
            .ok_or_else(|| AuthError::database("Invalid temp path".to_string()))?;

        // Keep temp dir alive for the duration of the test
        let db = SqliteUserDb::new(path).await?;
        // Note: temp_dir is implicitly kept alive in this scope
        std::mem::forget(temp_dir);
        Ok(db)
    }

    #[tokio::test]
    async fn test_create_and_get_user() {
        let db = test_db().await.unwrap();
        let user = UserRecord::new("alice", "hash123")
            .with_groups(vec!["admins", "users"]);

        db.create_user(user.clone()).await.unwrap();

        let fetched = db.get_user("alice").await.unwrap();
        assert_eq!(fetched.username, "alice");
        assert_eq!(fetched.password_hash, "hash123");
        assert_eq!(fetched.groups, vec!["admins", "users"]);
    }

    #[tokio::test]
    async fn test_user_not_found() {
        let db = test_db().await.unwrap();
        assert!(db.get_user("nonexistent").await.is_err());
    }

    #[tokio::test]
    async fn test_duplicate_user() {
        let db = test_db().await.unwrap();
        let user = UserRecord::new("alice", "hash");

        db.create_user(user.clone()).await.unwrap();
        assert!(db.create_user(user).await.is_err());
    }

    #[tokio::test]
    async fn test_update_password() {
        let db = test_db().await.unwrap();
        let user = UserRecord::new("alice", "old_hash");

        db.create_user(user).await.unwrap();
        db.update_password("alice", "new_hash".to_string()).await.unwrap();

        let fetched = db.get_user("alice").await.unwrap();
        assert_eq!(fetched.password_hash, "new_hash");
    }

    #[tokio::test]
    async fn test_update_password_nonexistent() {
        let db = test_db().await.unwrap();
        assert!(db.update_password("nonexistent", "hash".to_string()).await.is_err());
    }

    #[tokio::test]
    async fn test_list_users() {
        let db = test_db().await.unwrap();

        db.create_user(UserRecord::new("alice", "hash1")).await.unwrap();
        db.create_user(UserRecord::new("bob", "hash2")).await.unwrap();
        db.create_user(UserRecord::new("charlie", "hash3")).await.unwrap();

        let users = db.list_users().await.unwrap();
        assert_eq!(users.len(), 3);
        assert_eq!(users[0].username, "alice");
        assert_eq!(users[1].username, "bob");
        assert_eq!(users[2].username, "charlie");
    }

    #[tokio::test]
    async fn test_delete_user() {
        let db = test_db().await.unwrap();
        let user = UserRecord::new("alice", "hash");

        db.create_user(user).await.unwrap();
        db.delete_user("alice").await.unwrap();

        assert!(db.get_user("alice").await.is_err());
    }

    #[tokio::test]
    async fn test_delete_nonexistent() {
        let db = test_db().await.unwrap();
        assert!(db.delete_user("nonexistent").await.is_err());
    }

    #[tokio::test]
    async fn test_update_groups() {
        let db = test_db().await.unwrap();
        let user = UserRecord::new("alice", "hash")
            .with_groups(vec!["users"]);

        db.create_user(user).await.unwrap();
        db.update_groups("alice", vec!["users".to_string(), "admins".to_string()]).await.unwrap();

        let fetched = db.get_user("alice").await.unwrap();
        assert_eq!(fetched.groups, vec!["users", "admins"]);
    }
}

//! Database abstraction layer.
//!
//! This module provides traits and types for user database operations.
//! Users can implement the `UserDatabase` trait to support custom storage backends.

pub mod models;

pub use models::{UserDatabase, UserRecord};

/// Module for SQLite-specific implementations.
/// Available when the `sqlite` feature is enabled.
pub mod sqlite;

pub use sqlite::SqliteUserDb;

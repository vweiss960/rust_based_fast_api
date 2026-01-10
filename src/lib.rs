#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

//! # poem_auth - Authentication Framework for Poem
//!
//! A comprehensive, extensible authentication and authorization framework for the Poem web framework.
//!
//! ## Features
//!
//! - **Pluggable authentication architecture** - Easily add new auth methods
//! - **Multiple built-in providers** - Local database and LDAP/Active Directory
//! - **JWT token management** - Secure token generation and validation
//! - **Admin APIs** - User management and configuration endpoints
//! - **Audit logging** - Track authentication events
//! - **Rate limiting** - Protect against brute force attacks
//! - **Secure by default** - Argon2 password hashing, keyring secrets management
//!
//! ## Quick Start
//!
//! ### Basic Local Authentication
//!
//! ```rust,ignore
//! use poem::{listener::TcpListener, App, Route, get};
//! use poem_auth::prelude::*;
//! use poem_auth::db::sqlite::SqliteUserDb;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Setup database
//!     let db = SqliteUserDb::new("users.db").await?;
//!     let provider = LocalAuthProvider::new(db);
//!
//!     // Setup JWT middleware
//!     let jwt_auth = JwtAuth::new(JwtValidator::new("your-secret")?);
//!
//!     // Build app
//!     let app = App::new()
//!         .at("/protected", get(handler).with(jwt_auth));
//!
//!     // Run
//!     app.run(TcpListener::bind("127.0.0.1:3000")).await?;
//!     Ok(())
//! }
//!
//! async fn handler(claims: UserClaims) -> String {
//!     format!("Hello, {}!", claims.sub)
//! }
//! ```
//!
//! ## Architecture
//!
//! The crate is built around several key abstractions:
//!
//! - **`AuthProvider`** - The main extension point for custom authentication methods
//! - **`UserDatabase`** - Abstraction for user storage (SQLite provided by default)
//! - **`UserClaims`** - User information included in JWT tokens
//! - **`JwtAuth`** - Poem middleware for JWT validation
//!
//! ## Extensibility
//!
//! Implement your own authentication methods by implementing the `AuthProvider` trait:
//!
//! ```rust,ignore
//! use poem_auth::AuthProvider;
//! use async_trait::async_trait;
//!
//! struct MyOAuth2Provider { /* ... */ }
//!
//! #[async_trait]
//! impl AuthProvider for MyOAuth2Provider {
//!     async fn authenticate(&self, username: &str, password: &str)
//!         -> Result<UserClaims, AuthError> {
//!         // Your auth logic
//!         todo!()
//!     }
//!
//!     fn name(&self) -> &str {
//!         "oauth2"
//!     }
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - **`sqlite`** (default) - SQLite user database support
//! - **`ldap`** (default) - LDAP/Active Directory support
//! - **`keyring-support`** (default) - OS keyring integration for secrets
//! - **`cache`** (default) - In-memory token caching
//! - **`rate-limit`** - Rate limiting middleware
//! - **`cors`** - CORS support
//! - **`all`** - Enable all features
//!
//! ## Security
//!
//! This crate follows security best practices:
//!
//! - Passwords are hashed with **Argon2id**
//! - JWT secrets are stored in **OS keyrings** (with env var fallback)
//! - All authentication attempts are **logged**
//! - Tokens have **configurable expiration**
//! - Rate limiting protects against **brute force attacks**
//!
//! ## Documentation
//!
//! For detailed information, see:
//! - [`auth::AuthProvider`] - The main authentication trait
//! - [`auth::UserClaims`] - User information structure
//! - [`db::UserDatabase`] - Database abstraction
//! - [`middleware`] - Poem middleware components

pub mod auth;
pub mod db;
pub mod error;
pub mod password;
pub mod jwt;
pub mod middleware;
pub mod api;

// Providers
pub mod providers;

// Re-export commonly used types
pub use auth::{AuthProvider, UserClaims};
pub use db::{UserDatabase, UserRecord};
#[cfg(feature = "sqlite")]
pub use db::SqliteUserDb;
pub use error::{AuthError, ConfigError, SecretsError};
pub use providers::LocalAuthProvider;
#[cfg(feature = "ldap")]
pub use providers::{LdapAuthProvider, LdapConfig};
pub use password::{hash_password, verify_password};
pub use jwt::{JwtValidator, Token, TokenCache};
pub use middleware::{extract_jwt_claims, MasterAuth, MasterCredentials};
pub use api::types::{LoginRequest, LoginResponse, CreateUserRequest, UpdatePasswordRequest, ErrorResponse, UserClaimsResponse};

/// Prelude with commonly used imports.
///
/// # Example
///
/// ```rust,ignore
/// use poem_auth::prelude::*;
/// ```
pub mod prelude {
    pub use crate::auth::{AuthProvider, UserClaims};
    pub use crate::db::{UserDatabase, UserRecord};
    pub use crate::error::AuthError;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_library_compiles() {
        // This test just ensures the library compiles successfully
    }
}

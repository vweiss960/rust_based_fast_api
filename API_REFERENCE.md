# poem_auth API Reference

Complete API reference for all types and methods in the poem_auth crate.

## Table of Contents

- [Core Types](#core-types)
- [Authentication Providers](#authentication-providers)
- [Database Abstractions](#database-abstractions)
- [JWT & Token Management](#jwt--token-management)
- [Middleware](#middleware)
- [API Request/Response Types](#api-requestresponse-types)
- [Error Types](#error-types)
- [Utilities](#utilities)

## Core Types

### `UserClaims`

Represents authenticated user information in JWT tokens.

#### Fields

```rust
pub struct UserClaims {
    pub sub: String,                    // Username/user ID
    pub groups: Vec<String>,            // User's roles/groups
    pub provider: String,               // Auth method ("local", "ldap", etc.)
    pub exp: i64,                       // Expiration timestamp
    pub iat: i64,                       // Issued-at timestamp
    pub jti: String,                    // Unique token ID
    pub extra: Option<serde_json::Value>, // Custom claims
}
```

#### Methods

```rust
// Create new claims
pub fn new(username: &str, provider: &str, exp: i64, iat: i64) -> Self

// Add groups
pub fn with_groups<S: Into<String>>(self, groups: Vec<S>) -> Self
pub fn add_group<S: Into<String>>(self, group: S) -> Self

// Add custom claims
pub fn with_extra(self, extra: serde_json::Value) -> Self

// Check groups
pub fn has_group(&self, group: &str) -> bool
pub fn has_any_group(&self, groups: &[&str]) -> bool
pub fn has_all_groups(&self, groups: &[&str]) -> bool

// Token lifetime
pub fn is_expired(&self, now: i64) -> bool
pub fn time_to_expiry(&self, now: i64) -> i64
pub fn age(&self, now: i64) -> i64
```

#### Example

```rust
let claims = UserClaims::new("alice", "local", 1704067200, 1703980800)
    .with_groups(vec!["users", "admins"])
    .with_extra(serde_json::json!({"dept": "Engineering"}));

assert!(claims.has_group("users"));
assert!(claims.has_any_group(&["users", "guests"]));
```

---

## Authentication Providers

### `AuthProvider` Trait

Core trait for implementing authentication methods.

```rust
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Authenticate user with username and password
    ///
    /// Returns UserClaims on success, AuthError on failure
    async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<UserClaims, AuthError>;

    /// Return the provider's name (e.g., "local", "ldap")
    fn name(&self) -> &str;

    /// Validate provider configuration
    /// Called on startup to verify settings
    async fn validate_config(&self) -> Result<(), AuthError>;

    /// Return description of this provider
    fn info(&self) -> String;
}
```

### `LocalAuthProvider`

Authenticates against users stored in a database.

```rust
pub struct LocalAuthProvider { /* ... */ }

impl LocalAuthProvider {
    /// Create with a database
    pub fn new<D: UserDatabase + 'static>(db: D) -> Self

    /// Create with Arc-wrapped database
    pub fn with_db(db: Arc<dyn UserDatabase>) -> Self
}
```

#### Example

```rust
use poem_auth::providers::LocalAuthProvider;
use poem_auth::db::sqlite::SqliteUserDb;

let db = SqliteUserDb::new("users.db").await?;
let provider = LocalAuthProvider::new(db);

let claims = provider.authenticate("alice", "password123").await?;
```

### `LdapAuthProvider`

Authenticates against LDAP/Active Directory servers (requires `ldap` feature).

```rust
pub struct LdapAuthProvider { /* ... */ }

pub struct LdapConfig {
    pub server_url: String,              // ldap://host:port
    pub bind_dn_template: String,        // uid={username},ou=people,dc=example,dc=com
    pub user_search_base: String,        // ou=people,dc=example,dc=com
    pub user_object_class: String,       // inetOrgPerson
    pub group_search_base: String,       // ou=groups,dc=example,dc=com
    pub group_object_class: String,      // groupOfUniqueNames
    pub group_member_attribute: String,  // uniqueMember
    pub username_attribute: String,      // uid
    pub timeout_seconds: u64,            // Connection timeout
}

impl LdapAuthProvider {
    /// Create with configuration
    pub fn new(config: LdapConfig) -> Result<Self, AuthError>
}
```

#### Example

```rust
use poem_auth::providers::{LdapAuthProvider, LdapConfig};

let config = LdapConfig {
    server_url: "ldap://ldap.example.com:389".to_string(),
    bind_dn_template: "uid={username},ou=people,dc=example,dc=com".to_string(),
    user_search_base: "ou=people,dc=example,dc=com".to_string(),
    user_object_class: "inetOrgPerson".to_string(),
    group_search_base: "ou=groups,dc=example,dc=com".to_string(),
    group_object_class: "groupOfUniqueNames".to_string(),
    group_member_attribute: "uniqueMember".to_string(),
    username_attribute: "uid".to_string(),
    timeout_seconds: 10,
};

let provider = LdapAuthProvider::new(config)?;
let claims = provider.authenticate("jdoe", "password").await?;
```

---

## Database Abstractions

### `UserDatabase` Trait

Abstract interface for user storage backends.

```rust
#[async_trait]
pub trait UserDatabase: Send + Sync {
    /// Get user by username
    async fn get_user(&self, username: &str) -> Result<UserRecord, AuthError>;

    /// Create new user
    async fn create_user(&self, user: UserRecord) -> Result<(), AuthError>;

    /// Update user's password
    async fn update_password(&self, username: &str, hash: &str) -> Result<(), AuthError>;

    /// List all users
    async fn list_users(&self) -> Result<Vec<UserRecord>, AuthError>;

    /// Delete user
    async fn delete_user(&self, username: &str) -> Result<(), AuthError>;

    /// Default implementation: check if user exists
    async fn user_exists(&self, username: &str) -> bool { /* ... */ }

    /// Default implementation: update user's groups
    async fn update_groups(
        &self,
        username: &str,
        groups: Vec<String>,
    ) -> Result<(), AuthError> { /* ... */ }
}
```

### `UserRecord`

Represents a user record in the database.

```rust
pub struct UserRecord {
    pub username: String,
    pub password_hash: String,
    pub groups: Vec<String>,
    pub enabled: bool,
}

impl UserRecord {
    /// Create new user
    pub fn new(username: &str, password_hash: &str) -> Self

    /// Add groups
    pub fn with_groups<S: Into<String>>(self, groups: Vec<S>) -> Self
    pub fn add_group<S: Into<String>>(self, group: S) -> Self

    /// Enable/disable user
    pub fn enable(self) -> Self
    pub fn disable(self) -> Self
    pub fn with_enabled(self, enabled: bool) -> Self
}
```

#### Example

```rust
use poem_auth::db::UserRecord;
use poem_auth::password::hash_password;

let hash = hash_password("password123")?;
let user = UserRecord::new("alice", &hash)
    .with_groups(vec!["users", "developers"])
    .enable();

db.create_user(user).await?;
```

### `SqliteUserDb`

SQLite implementation of `UserDatabase` (requires `sqlite` feature).

```rust
pub struct SqliteUserDb { /* ... */ }

impl SqliteUserDb {
    /// Create new SQLite database
    /// Auto-creates tables and initializes schema
    pub async fn new(path: &str) -> Result<Self, AuthError>

    /// Get the underlying connection pool
    pub fn pool(&self) -> &SqlitePool
}
```

#### Example

```rust
use poem_auth::db::sqlite::SqliteUserDb;

let db = SqliteUserDb::new("users.db").await?;
let user = db.get_user("alice").await?;
let users = db.list_users().await?;
```

---

## JWT & Token Management

### `JwtValidator`

Handles JWT token generation and validation.

```rust
pub struct JwtValidator { /* ... */ }

impl JwtValidator {
    /// Create validator with secret key
    /// Secret must be at least 16 characters
    pub fn new(secret: &str) -> Result<Self, AuthError>

    /// Encode claims into JWT token
    pub fn encode(&self, claims: &UserClaims) -> Result<String, AuthError>

    /// Decode and validate JWT token
    pub fn decode(&self, token: &str) -> Result<UserClaims, AuthError>
}
```

#### Example

```rust
use poem_auth::jwt::JwtValidator;
use poem_auth::UserClaims;

let validator = JwtValidator::new("my-super-secret-key-min-16-chars")?;

let claims = UserClaims::new("alice", "local", 1704067200, 1703980800);
let token = validator.encode(&claims)?;

let decoded = validator.decode(&token)?;
assert_eq!(decoded.sub, "alice");
```

### `TokenCache`

In-memory token caching (requires `cache` feature).

```rust
pub struct TokenCache { /* ... */ }

impl TokenCache {
    /// Create cache with TTL
    pub fn new(ttl: Duration) -> Self

    /// Get cached token (if not expired)
    pub async fn get(&self, token: &str) -> Option<UserClaims>

    /// Cache a token
    pub async fn insert(&self, token: String, claims: UserClaims)

    /// Clear all cached tokens
    pub async fn clear(&self)
}
```

#### Example

```rust
use poem_auth::jwt::TokenCache;
use std::time::Duration;

let cache = TokenCache::new(Duration::from_secs(300));

// Cache a token
cache.insert("token123".to_string(), claims).await;

// Retrieve from cache
if let Some(cached_claims) = cache.get("token123").await {
    println!("Found in cache: {}", cached_claims.sub);
}
```

### `Token` Type

```rust
pub type Token = String;
```

---

## Middleware

### `extract_jwt_claims`

Helper function to extract and validate JWT from Bearer token.

```rust
pub async fn extract_jwt_claims(
    req: &Request,
    validator: &JwtValidator,
) -> Result<UserClaims, AuthError>
```

#### Example

```rust
use poem_auth::middleware::extract_jwt_claims;

async fn my_handler(req: &Request) -> Result<String, String> {
    let claims = extract_jwt_claims(req, &validator).await
        .map_err(|e| format!("Auth failed: {}", e))?;
    Ok(format!("Hello {}", claims.sub))
}
```

### `MasterAuth`

Validates master admin password (separate from JWT).

```rust
pub struct MasterAuth;
pub struct MasterCredentials;

impl MasterAuth {
    /// Validate master password from header
    pub async fn validate(req: &Request) -> Result<MasterCredentials, AuthError>
}
```

#### Example

```rust
use poem::{post, StatusCode};
use poem_auth::middleware::MasterAuth;

#[post("/admin/reset")]
async fn admin_reset(_auth: MasterAuth) -> StatusCode {
    StatusCode::OK
}
```

### `RateLimit` and `RateLimitConfig`

Rate limiting by IP address (requires `rate-limit` feature).

```rust
pub struct RateLimitConfig {
    pub general_requests_per_second: u32,    // Default: 100
    pub auth_requests_per_second: u32,       // Default: 5
}

pub struct RateLimit { /* ... */ }

impl RateLimit {
    /// Create rate limiter with config
    pub fn new(config: RateLimitConfig) -> Self

    /// Check if request is allowed
    pub async fn check(&self, ip: &str) -> Result<(), AuthError>
}
```

#### Example

```rust
use poem_auth::middleware::{RateLimit, RateLimitConfig};

let limiter = RateLimit::new(RateLimitConfig {
    general_requests_per_second: 100,
    auth_requests_per_second: 5,
});

// Check if request is allowed
match limiter.check("192.168.1.1").await {
    Ok(()) => {
        // Process request
    }
    Err(AuthError::RateLimitExceeded) => {
        // Return 429 Too Many Requests
    }
    Err(e) => {
        // Other error
    }
}
```

---

## API Request/Response Types

### `LoginRequest`

```rust
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub provider: Option<String>,  // "local", "ldap", etc. (optional)
}
```

#### Example

```rust
let req = LoginRequest {
    username: "alice".to_string(),
    password: "password123".to_string(),
    provider: Some("local".to_string()),
};
```

### `LoginResponse`

```rust
pub struct LoginResponse {
    pub token: String,              // JWT token
    pub token_type: String,         // Always "Bearer"
    pub expires_in: i64,            // Seconds until expiration
    pub claims: UserClaimsResponse,
}
```

### `UserClaimsResponse`

Simplified claims for API responses.

```rust
pub struct UserClaimsResponse {
    pub sub: String,
    pub provider: String,
    pub groups: Vec<String>,
    pub exp: i64,
    pub iat: i64,
}

impl UserClaimsResponse {
    /// Convert from UserClaims
    pub fn from_claims(claims: UserClaims) -> Self
}
```

### `CreateUserRequest`

```rust
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub groups: Vec<String>,        // Optional, defaults to empty
    pub enabled: bool,              // Optional, defaults to true
}
```

### `UpdatePasswordRequest`

```rust
pub struct UpdatePasswordRequest {
    pub username: String,
    pub new_password: String,
}
```

### `UserResponse`

```rust
pub struct UserResponse {
    pub username: String,
    pub enabled: bool,
    pub groups: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### `ErrorResponse`

```rust
pub struct ErrorResponse {
    pub error: String,              // Machine-readable code
    pub message: String,            // Human-readable message
    pub details: Option<String>,    // Optional additional info
}

impl ErrorResponse {
    // Constructors
    pub fn new(error: &str, message: &str) -> Self
    pub fn with_details(error: &str, message: &str, details: &str) -> Self

    // Helpers
    pub fn invalid_credentials() -> Self
    pub fn user_not_found(username: &str) -> Self
    pub fn user_disabled(username: &str) -> Self
    pub fn unauthorized() -> Self
    pub fn forbidden(reason: &str) -> Self
}
```

#### Example

```rust
use poem_auth::api::types::ErrorResponse;

let error = ErrorResponse::invalid_credentials();
let error = ErrorResponse::user_not_found("alice");
let error = ErrorResponse::with_details(
    "invalid_input",
    "Username must be alphanumeric",
    "Provided username: '!@#$'"
);
```

---

## Error Types

### `AuthError`

```rust
pub enum AuthError {
    InvalidCredentials,
    UserNotFound,
    UserDisabled,
    InvalidToken,
    TokenExpired,
    RateLimitExceeded,
    DatabaseError(String),
    ConfigError(String),
    JwtError(String),
    LdapError(String),
    SecretsError(String),
}

impl AuthError {
    // Type checks
    pub fn is_invalid_credentials(&self) -> bool
    pub fn is_token_error(&self) -> bool
    pub fn is_database_error(&self) -> bool

    // Constructors
    pub fn database(msg: impl Into<String>) -> Self
    pub fn config(msg: impl Into<String>) -> Self
    pub fn jwt(msg: impl Into<String>) -> Self
    pub fn ldap(msg: impl Into<String>) -> Self
    pub fn secrets(msg: impl Into<String>) -> Self
}
```

#### Example

```rust
use poem_auth::error::AuthError;

match provider.authenticate("user", "pass").await {
    Ok(claims) => {},
    Err(AuthError::InvalidCredentials) => println!("Wrong password"),
    Err(AuthError::UserNotFound) => println!("User not found"),
    Err(AuthError::UserDisabled) => println!("Account disabled"),
    Err(AuthError::RateLimitExceeded) => println!("Too many attempts"),
    Err(e) => println!("Error: {}", e),
}
```

### `ConfigError`

```rust
pub struct ConfigError {
    pub message: String,
    pub source: Option<String>,
}
```

### `SecretsError`

```rust
pub enum SecretsError {
    NotFound(String),
    StoreError(String),
}
```

---

## Utilities

### Password Hashing

```rust
pub use crate::password::{hash_password, verify_password};

/// Hash password with Argon2id
pub fn hash_password(password: &str) -> Result<String, AuthError>

/// Verify password against hash (constant-time)
pub fn verify_password(password: &str, hash: &str) -> Result<(), AuthError>
```

#### Example

```rust
use poem_auth::password::{hash_password, verify_password};

let hash = hash_password("my_password")?;

// Later, verify:
match verify_password("my_password", &hash) {
    Ok(()) => println!("Password correct"),
    Err(_) => println!("Password incorrect"),
}
```

### Prelude

Commonly used types exported for convenience:

```rust
pub mod prelude {
    pub use crate::auth::{AuthProvider, UserClaims};
    pub use crate::db::{UserDatabase, UserRecord};
    pub use crate::error::AuthError;
}
```

#### Example

```rust
use poem_auth::prelude::*;

// Can now use:
// - UserClaims
// - UserDatabase
// - UserRecord
// - AuthError
// - AuthProvider
```

---

## Common Patterns

### Implementing Custom AuthProvider

```rust
use async_trait::async_trait;
use poem_auth::{AuthProvider, UserClaims};
use poem_auth::error::AuthError;

struct MyCustomProvider;

#[async_trait]
impl AuthProvider for MyCustomProvider {
    async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<UserClaims, AuthError> {
        // Your authentication logic
        let now = chrono::Utc::now().timestamp();
        Ok(UserClaims::new(username, "custom", now + 86400, now))
    }

    fn name(&self) -> &str {
        "custom"
    }

    async fn validate_config(&self) -> Result<(), AuthError> {
        Ok(())
    }

    fn info(&self) -> String {
        "Custom authentication provider".to_string()
    }
}
```

### Implementing Custom Database

```rust
use async_trait::async_trait;
use poem_auth::{UserDatabase, UserRecord};
use poem_auth::error::AuthError;

struct MyDatabase;

#[async_trait]
impl UserDatabase for MyDatabase {
    async fn get_user(&self, username: &str) -> Result<UserRecord, AuthError> {
        // Fetch from your backend
        todo!()
    }

    async fn create_user(&self, user: UserRecord) -> Result<(), AuthError> {
        todo!()
    }

    async fn update_password(&self, username: &str, hash: &str) -> Result<(), AuthError> {
        todo!()
    }

    async fn list_users(&self) -> Result<Vec<UserRecord>, AuthError> {
        todo!()
    }

    async fn delete_user(&self, username: &str) -> Result<(), AuthError> {
        todo!()
    }
}
```

---

## Type Aliases

```rust
pub type Token = String;
```

---

## Feature Flags

| Feature | Default | Purpose |
|---------|---------|---------|
| `sqlite` | Yes | SQLite database support |
| `ldap` | No | LDAP/AD provider |
| `cache` | Yes | Token caching |
| `rate-limit` | No | Rate limiting |
| `cors` | No | CORS support |
| `cli` | No | CLI utilities |

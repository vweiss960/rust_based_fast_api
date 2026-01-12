# poem_auth

A comprehensive, extensible authentication and authorization framework for the Poem web framework.

## Features

- **Pluggable authentication architecture** - Easily add new auth methods (OAuth2, SAML, etc.)
- **Multiple built-in providers** - Local database and LDAP/Active Directory support
- **JWT token management** - Secure token generation, validation, and caching
- **Admin APIs** - User management and configuration endpoints
- **Audit logging** - Track authentication events for compliance
- **Rate limiting** - Protect against brute force attacks
- **Secure by default** - Argon2 password hashing, keyring secrets management

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
- [Usage Guides](#usage-guides)
  - [Local Authentication](#local-authentication)
  - [LDAP/Active Directory](#ldapactive-directory)
  - [Custom Authentication Providers](#custom-authentication-providers)
- [JWT & Token Management](#jwt--token-management)
- [Middleware Integration](#middleware-integration)
- [User Management APIs](#user-management-apis)
- [Advanced Topics](#advanced-topics)
- [Feature Flags](#feature-flags)
- [Security](#security)
- [License](#license)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
poem = { version = "3", features = ["tower"] }
poem_auth = { version = "0.1", features = ["sqlite"] }
tokio = { version = "1", features = ["full"] }
```

### Feature Selection

**Basic setup** (local database only):
```toml
poem_auth = { version = "0.1", features = ["sqlite"] }
```

**With LDAP support**:
```toml
poem_auth = { version = "0.1", features = ["sqlite", "ldap"] }
```

**Full featured**:
```toml
poem_auth = { version = "0.1", features = ["sqlite", "ldap", "cache", "rate-limit", "cors"] }
```

## Quick Start

A minimal example with local database authentication:

```rust
use poem::{listener::TcpListener, App, get, Route};
use poem_auth::prelude::*;
use poem_auth::db::sqlite::SqliteUserDb;
use poem_auth::providers::LocalAuthProvider;
use poem_auth::jwt::JwtValidator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let db = SqliteUserDb::new("users.db").await?;

    // Create authentication provider
    let auth_provider = LocalAuthProvider::new(db);

    // Initialize JWT validator (use a real secret in production!)
    let jwt = JwtValidator::new("your-256-bit-secret-key-min-16-chars")?;

    // Build your app
    let app = App::new()
        .at("/public", get(public_handler))
        .at("/protected", get(protected_handler));

    // Run the server
    app.run(TcpListener::bind("127.0.0.1:3000")).await?;
    Ok(())
}

async fn public_handler() -> String {
    "This is public".to_string()
}

async fn protected_handler(claims: UserClaims) -> String {
    format!("Hello, {}!", claims.sub)
}
```

## Core Concepts

### UserClaims

The `UserClaims` struct represents authenticated user information contained in JWT tokens.

```rust
use poem_auth::UserClaims;

// Claims are automatically created by auth providers during login
let claims = UserClaims::new("alice", "local", 1704067200, 1703980800)
    .with_groups(vec!["admins", "users"])
    .with_extra(serde_json::json!({"department": "Engineering"}));

// Check user information
assert_eq!(claims.sub, "alice");  // username
assert!(claims.has_group("admins"));
assert!(!claims.is_expired(1703980900));
println!("Token age: {} seconds", claims.age(1703980900));
```

**Key fields:**
- `sub` - Username/user ID
- `provider` - Authentication method used (e.g., "local", "ldap")
- `groups` - User's roles/groups for authorization
- `exp` - Unix timestamp when token expires
- `iat` - Unix timestamp when token was issued
- `jti` - Unique token ID (for revocation tracking)
- `extra` - Additional custom claims

### AuthProvider Trait

The core abstraction for implementing authentication methods:

```rust
use async_trait::async_trait;
use poem_auth::{AuthProvider, UserClaims};
use poem_auth::error::AuthError;

#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Authenticate a user with credentials
    async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<UserClaims, AuthError>;

    /// Return a name for this provider
    fn name(&self) -> &str;

    /// Validate provider configuration
    async fn validate_config(&self) -> Result<(), AuthError>;

    /// Return info about this provider
    fn info(&self) -> String;
}
```

### UserDatabase Trait

Abstraction for user storage backends:

```rust
use async_trait::async_trait;
use poem_auth::{UserDatabase, UserRecord};
use poem_auth::error::AuthError;

#[async_trait]
pub trait UserDatabase: Send + Sync {
    /// Get a user by username
    async fn get_user(&self, username: &str) -> Result<UserRecord, AuthError>;

    /// Create a new user
    async fn create_user(&self, user: UserRecord) -> Result<(), AuthError>;

    /// Update user's password
    async fn update_password(&self, username: &str, hash: &str) -> Result<(), AuthError>;

    /// List all users
    async fn list_users(&self) -> Result<Vec<UserRecord>, AuthError>;

    /// Delete a user
    async fn delete_user(&self, username: &str) -> Result<(), AuthError>;
}
```

## Usage Guides

### Local Authentication

Local authentication stores users in a database with Argon2-hashed passwords.

#### Setting Up Local Auth

```rust
use poem_auth::db::sqlite::SqliteUserDb;
use poem_auth::providers::LocalAuthProvider;
use poem_auth::db::UserRecord;
use poem_auth::password::hash_password;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database
    let db = SqliteUserDb::new("users.db").await?;

    // Create a test user
    let password_hash = hash_password("secure_password")?;
    let user = UserRecord::new("alice", &password_hash)
        .with_groups(vec!["users", "developers"]);
    db.create_user(user).await?;

    // Create provider
    let provider = LocalAuthProvider::new(db);

    // Authenticate
    let claims = provider.authenticate("alice", "secure_password").await?;
    println!("Authenticated as: {}", claims.sub);

    Ok(())
}
```

#### Creating Users at Runtime

```rust
use poem::{post, Body};
use poem_auth::api::types::{CreateUserRequest, ErrorResponse};
use poem_auth::db::UserDatabase;
use poem_auth::password::hash_password;
use poem_auth::db::UserRecord;

#[post("/admin/users")]
async fn create_user(
    req: CreateUserRequest,
    db: &dyn UserDatabase,
) -> Result<String, ErrorResponse> {
    // Validate username isn't already taken
    if db.get_user(&req.username).await.is_ok() {
        return Err(ErrorResponse::new("user_exists", "Username already taken"));
    }

    // Hash password
    let password_hash = hash_password(&req.password)
        .map_err(|_| ErrorResponse::new("error", "Failed to hash password"))?;

    // Create user
    let user = UserRecord::new(&req.username, &password_hash)
        .with_groups(req.groups)
        .with_enabled(req.enabled);

    db.create_user(user).await
        .map_err(|_| ErrorResponse::new("error", "Failed to create user"))?;

    Ok(format!("User '{}' created", req.username))
}
```

#### Managing User Groups

```rust
use poem_auth::UserClaims;

// Check user permissions
fn check_permission(claims: &UserClaims, required_groups: &[&str]) -> bool {
    claims.has_any_group(required_groups)
}

// Route handler with role check
async fn admin_handler(claims: UserClaims) -> Result<String, String> {
    if !claims.has_group("admins") {
        return Err("Admin access required".to_string());
    }
    Ok("Admin panel accessed".to_string())
}
```

### LDAP/Active Directory

Authenticate users against an LDAP or Active Directory server.

#### Setting Up LDAP

```rust
use poem_auth::providers::{LdapAuthProvider, LdapConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure LDAP connection
    let ldap_config = LdapConfig {
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

    // Create LDAP provider
    let ldap_provider = LdapAuthProvider::new(ldap_config)?;

    // Validate LDAP connectivity on startup
    ldap_provider.validate_config().await?;

    // Authenticate user
    let claims = ldap_provider.authenticate("jdoe", "password").await?;
    println!("Authenticated LDAP user: {} with groups: {:?}",
             claims.sub, claims.groups);

    Ok(())
}
```

#### Using LDAP with Fallback to Local Auth

```rust
use poem_auth::error::AuthError;
use poem_auth::{AuthProvider, UserClaims};

// Try LDAP first, fall back to local database
async fn authenticate_with_fallback(
    username: &str,
    password: &str,
    ldap: &LdapAuthProvider,
    local: &LocalAuthProvider,
) -> Result<UserClaims, AuthError> {
    // Try LDAP first
    match ldap.authenticate(username, password).await {
        Ok(claims) => return Ok(claims),
        Err(_) => {
            // Fall back to local auth
            return local.authenticate(username, password).await;
        }
    }
}
```

### Custom Authentication Providers

Implement the `AuthProvider` trait for custom authentication methods (OAuth2, SAML, etc.):

```rust
use async_trait::async_trait;
use poem_auth::{AuthProvider, UserClaims};
use poem_auth::error::AuthError;

struct MyOAuth2Provider {
    client_id: String,
    client_secret: String,
}

#[async_trait]
impl AuthProvider for MyOAuth2Provider {
    async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<UserClaims, AuthError> {
        // In OAuth2 flows, username might be a code and password might be unused
        // This is just an example structure

        // Your OAuth2 verification logic here
        let now = chrono::Utc::now().timestamp();
        let exp = now + 86400; // 24 hours

        Ok(UserClaims::new(username, "oauth2", exp, now)
            .with_groups(vec!["oauth_users"]))
    }

    fn name(&self) -> &str {
        "oauth2"
    }

    async fn validate_config(&self) -> Result<(), AuthError> {
        if self.client_id.is_empty() || self.client_secret.is_empty() {
            return Err(AuthError::config("Missing OAuth2 credentials"));
        }
        Ok(())
    }

    fn info(&self) -> String {
        "OAuth2 authentication provider".to_string()
    }
}
```

## JWT & Token Management

### Token Generation and Validation

```rust
use poem_auth::jwt::JwtValidator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create validator with secret key
    // Use at least 16 characters for the secret
    let validator = JwtValidator::new("your-super-secret-key-min-16-chars")?;

    // Generate token from claims
    let claims = UserClaims::new("alice", "local", 1704067200, 1703980800);
    let token = validator.encode(&claims)?;
    println!("Generated token: {}", token);

    // Validate and decode token
    let decoded_claims = validator.decode(&token)?;
    println!("Decoded claims: {:?}", decoded_claims);

    Ok(())
}
```

### Token Caching (Optional)

When the `cache` feature is enabled, tokens are cached in-memory to reduce verification overhead:

```rust
use poem_auth::jwt::{JwtValidator, TokenCache};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let validator = JwtValidator::new("secret")?;

    // Create cache with custom TTL
    let cache = TokenCache::new(std::time::Duration::from_secs(300));

    // Cache stores decoded tokens to avoid re-verification
    let token = validator.encode(&claims)?;

    // First decode - goes to validator
    let claims1 = validator.decode(&token)?;

    // Subsequent decodes in cache window - uses cache
    let claims2 = validator.decode(&token)?;

    Ok(())
}
```

## Middleware Integration

### JWT Authentication Middleware

Protect routes by verifying JWT tokens:

```rust
use poem::{middleware::Next, Request, Response, get};
use poem_auth::middleware::extract_jwt_claims;
use poem_auth::UserClaims;

#[get("/protected")]
async fn protected_route(claims: UserClaims) -> String {
    format!("Hello {}", claims.sub)
}

// The UserClaims extractor automatically:
// 1. Looks for Authorization: Bearer <token> header
// 2. Validates the token signature
// 3. Checks expiration
// 4. Injects claims into the handler
```

### Master Authentication

Protect admin endpoints with a separate master password:

```rust
use poem::{post, StatusCode};
use poem_auth::middleware::{MasterAuth, MasterCredentials};

#[post("/admin/reset")]
async fn admin_reset(
    _auth: MasterAuth,  // Automatically checks master password
) -> StatusCode {
    // Admin action performed
    StatusCode::OK
}

// Client must send:
// POST /admin/reset
// Authorization: Bearer <master-password>
```

### Rate Limiting

Prevent brute force attacks (requires `rate-limit` feature):

```rust
use poem_auth::middleware::{RateLimit, RateLimitConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure rate limits
    let rate_limit = RateLimitConfig {
        general_requests_per_second: 100,
        auth_requests_per_second: 5,
    };

    let limiter = RateLimit::new(rate_limit);

    // The limiter tracks requests by IP address and enforces limits
    // Auth endpoints (/login, /auth) have stricter limits

    Ok(())
}
```

## User Management APIs

### Login Endpoint

```rust
use poem::{post, web::Json};
use poem_auth::api::types::{LoginRequest, LoginResponse, ErrorResponse};
use poem_auth::{AuthProvider, UserClaims};
use poem_auth::jwt::JwtValidator;

#[post("/login")]
async fn login(
    req: Json<LoginRequest>,
    provider: &dyn AuthProvider,
    jwt: &JwtValidator,
) -> Result<Json<LoginResponse>, Json<ErrorResponse>> {
    // Authenticate user
    let claims = provider.authenticate(&req.username, &req.password)
        .await
        .map_err(|_| Json(ErrorResponse::invalid_credentials()))?;

    // Generate token
    let token = jwt.encode(&claims)
        .map_err(|_| Json(ErrorResponse::new("error", "Token generation failed")))?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: claims.exp - claims.iat,
        claims: UserClaimsResponse::from_claims(claims),
    }))
}
```

### User CRUD Operations

```rust
use poem::{get, post, delete, web::Path};
use poem_auth::db::{UserDatabase, UserRecord};
use poem_auth::api::types::{CreateUserRequest, UserResponse, ErrorResponse};
use poem_auth::password::hash_password;

#[post("/users")]
async fn create_user(
    req: CreateUserRequest,
    db: &dyn UserDatabase,
) -> Result<Json<UserResponse>, Json<ErrorResponse>> {
    let hash = hash_password(&req.password)?;
    let user = UserRecord::new(&req.username, &hash)
        .with_groups(req.groups);

    db.create_user(user).await?;
    Ok(Json(/* ... */))
}

#[get("/users/<username>")]
async fn get_user(
    username: Path<String>,
    db: &dyn UserDatabase,
) -> Result<Json<UserResponse>, Json<ErrorResponse>> {
    let user = db.get_user(&username).await?;
    Ok(Json(/* ... */))
}

#[delete("/users/<username>")]
async fn delete_user(
    username: Path<String>,
    db: &dyn UserDatabase,
) -> Result<StatusCode, Json<ErrorResponse>> {
    db.delete_user(&username).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

## Advanced Topics

### Customizing Token Expiration

```rust
use chrono::Utc;

// Create claims with custom expiration
let now = Utc::now().timestamp();
let exp = now + (7 * 24 * 60 * 60);  // 7 days

let claims = UserClaims::new("alice", "local", exp, now);
```

### Custom Claims

```rust
use serde_json::json;

let claims = UserClaims::new("alice", "local", exp, iat)
    .with_extra(json!({
        "department": "Engineering",
        "clearance_level": 5,
        "ip_address": "192.168.1.1"
    }));

// Access in handler
if let Some(extra) = &claims.extra {
    let dept = extra.get("department");
}
```

### Authorization Helpers

```rust
// Check single group
if claims.has_group("admins") {
    // Allow admin action
}

// Check any of multiple groups
if claims.has_any_group(&["admins", "moderators"]) {
    // Allow moderation
}

// Require all groups
if claims.has_all_groups(&["users", "verified"]) {
    // Allow action requiring both
}
```

### Error Handling

```rust
use poem_auth::error::AuthError;

match provider.authenticate("user", "pass").await {
    Ok(claims) => println!("Success: {}", claims.sub),
    Err(AuthError::InvalidCredentials) => println!("Wrong password"),
    Err(AuthError::UserNotFound) => println!("User doesn't exist"),
    Err(AuthError::UserDisabled) => println!("Account is disabled"),
    Err(e) => println!("Other error: {}", e),
}
```

## Feature Flags

- **`sqlite`** (default) - SQLite user database support
- **`ldap`** - LDAP/Active Directory support (requires OpenSSL)
- **`cache`** (default) - In-memory token caching with moka
- **`rate-limit`** - Rate limiting middleware
- **`cors`** - CORS support via tower-http
- **`cli`** - CLI utility for user management

Enable features in `Cargo.toml`:

```toml
# Just SQLite
poem_auth = { version = "0.1", features = ["sqlite"] }

# SQLite + LDAP + Rate limiting
poem_auth = { version = "0.1", features = ["sqlite", "ldap", "rate-limit"] }
```

## Security

This crate follows security best practices:

- **Password Hashing**: Passwords are hashed using **Argon2id** with secure defaults:
  - Memory: 19,456 KiB
  - Time cost: 2
  - Parallelism: 1

- **Token Security**:
  - JWT uses **HS256** signing
  - Secrets should be at least 16 characters
  - Tokens have configurable expiration (default 24 hours)

- **Audit Logging**: All authentication events are logged via `tracing`

- **Rate Limiting**: Protects authentication endpoints from brute force attacks

- **Input Validation**: Passwords must be 1-128 characters

- **Constant-Time Comparison**: Password verification uses constant-time comparison to prevent timing attacks

### Production Checklist

- [ ] Use a strong, random JWT secret (20+ characters)
- [ ] Enable rate limiting on authentication endpoints
- [ ] Configure appropriate token expiration for your use case
- [ ] Set up audit logging to track authentication events
- [ ] Use LDAP/AD with TLS in production
- [ ] Regularly rotate JWT secrets
- [ ] Enforce HTTPS for all authentication endpoints
- [ ] Implement token refresh for long-lived sessions

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

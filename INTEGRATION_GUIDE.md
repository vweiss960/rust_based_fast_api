# Integration Guide: Using poem_auth in Your Project

Complete guide on how to add and use the poem_auth crate in another Rust project.

## Table of Contents

- [Quick Integration](#quick-integration)
- [Installation Methods](#installation-methods)
- [Basic Setup](#basic-setup)
- [Feature Selection](#feature-selection)
- [Common Integration Patterns](#common-integration-patterns)
- [Configuration](#configuration)
- [Error Handling](#error-handling)
- [Testing](#testing)
- [Production Deployment](#production-deployment)
- [Troubleshooting](#troubleshooting)

---

## Quick Integration

### 1-Minute Setup

```toml
# In your Cargo.toml
[dependencies]
poem = { version = "3", features = ["tower"] }
poem_auth = { version = "0.1", path = "../rust_based_fast_api" }
tokio = { version = "1", features = ["full"] }
```

```rust
// In your main.rs or lib.rs
use poem::{listener::TcpListener, App, get};
use poem_auth::prelude::*;
use poem_auth::db::sqlite::SqliteUserDb;
use poem_auth::providers::LocalAuthProvider;
use poem_auth::jwt::JwtValidator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = SqliteUserDb::new("auth.db").await?;
    let provider = LocalAuthProvider::new(db);
    let jwt = JwtValidator::new("your-secret-key-min-16-chars")?;

    let app = App::new()
        .at("/protected", get(protected_handler));

    app.run(TcpListener::bind("127.0.0.1:3000")).await?;
    Ok(())
}

async fn protected_handler(claims: UserClaims) -> String {
    format!("Hello, {}", claims.sub)
}
```

---

## Installation Methods

### Method 1: Local Path Reference (Development)

Use this when developing locally or in a monorepo:

```toml
[dependencies]
poem_auth = { version = "0.1", path = "../rust_based_fast_api" }
```

**Pros:**
- Easy to modify and test
- Immediate updates without publishing
- Good for development

**Cons:**
- Relative paths can break if project structure changes
- Not suitable for published crates

**When to use:** Local development, monorepos, internal projects

---

### Method 2: Git Reference (Pre-Publication)

Use this when the crate isn't published to crates.io yet:

```toml
[dependencies]
poem_auth = { git = "https://github.com/yourusername/rust_based_fast_api", branch = "main" }
```

Or with a specific commit:

```toml
[dependencies]
poem_auth = { git = "https://github.com/yourusername/rust_based_fast_api", rev = "abc1234" }
```

**Pros:**
- Works with remote repositories
- Can use specific branches/commits
- Good for CI/CD pipelines

**Cons:**
- Slower compilation (downloads from git)
- Requires git access
- Not deterministic without rev/tag

**When to use:** CI/CD, sharing across teams, pre-publication

---

### Method 3: Published to crates.io (Production)

Once published to crates.io:

```toml
[dependencies]
poem_auth = "0.1"  # Uses latest 0.1.x
poem_auth = "0.1.0"  # Specific version
poem_auth = "^0.1.0"  # Compatible: 0.1.0 to <0.2.0
poem_auth = "~0.1.0"  # Relaxed: 0.1.0 to <0.1.∞
```

**Pros:**
- Standard Rust distribution method
- Fast compilation (pre-compiled if available)
- Version management via Cargo.lock
- Easy updates

**Cons:**
- Requires publication to crates.io
- Slower if waiting for binary caches

**When to use:** Published projects, public distribution

---

## Basic Setup

### Step 1: Add Dependencies

```toml
[dependencies]
poem = { version = "3", features = ["tower"] }
poem_auth = { version = "0.1", path = "../rust_based_fast_api" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### Step 2: Initialize Authentication

```rust
use poem_auth::prelude::*;
use poem_auth::db::sqlite::SqliteUserDb;
use poem_auth::providers::LocalAuthProvider;
use poem_auth::jwt::JwtValidator;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let db = Arc::new(SqliteUserDb::new("users.db").await?);

    // Create authentication provider
    let provider = Arc::new(LocalAuthProvider::new(db.as_ref().clone()));

    // Initialize JWT validator
    let jwt = Arc::new(JwtValidator::new("your-secret-key-minimum-16-characters")?);

    // Build your Poem app with auth
    build_app(db.clone(), provider.clone(), jwt.clone()).await?;

    Ok(())
}

async fn build_app(
    db: Arc<dyn UserDatabase>,
    provider: Arc<LocalAuthProvider>,
    jwt: Arc<JwtValidator>,
) -> Result<(), Box<dyn std::error::Error>> {
    use poem::{Route, listener::TcpListener, App, get, post};

    let app = App::new()
        .route(
            Route::new()
                .at("/login", post(login_handler))
                .at("/protected", get(protected_handler))
        );

    app.run(TcpListener::bind("127.0.0.1:3000")).await?;
    Ok(())
}
```

### Step 3: Create Route Handlers

```rust
use poem::{web::Json, http::StatusCode};
use poem_auth::api::types::{LoginRequest, LoginResponse, ErrorResponse, UserClaimsResponse};

async fn login_handler(
    req: Json<LoginRequest>,
    provider: &LocalAuthProvider,
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

async fn protected_handler(claims: UserClaims) -> String {
    format!("Hello, {}! You are authenticated.", claims.sub)
}
```

---

## Feature Selection

### Minimal Setup (Core Only)

```toml
[dependencies]
poem_auth = { version = "0.1", default-features = false, features = ["sqlite"] }
```

Includes:
- Local authentication ✓
- JWT tokens ✓
- SQLite database ✓
- Basic password hashing ✓

---

### Standard Setup (Recommended)

```toml
[dependencies]
poem_auth = { version = "0.1", features = ["sqlite", "cache"] }
```

Includes everything minimal + :
- Token caching (performance) ✓
- Better performance for high traffic ✓

---

### Full Setup (All Features)

```toml
[dependencies]
poem_auth = { version = "0.1", features = ["sqlite", "cache", "ldap", "rate-limit", "cors"] }
```

Includes everything + :
- LDAP/Active Directory support ✓
- Rate limiting middleware ✓
- CORS support ✓
- Enterprise features ✓

---

### Feature Combinations

```toml
# For microservices with rate limiting
poem_auth = { version = "0.1", features = ["sqlite", "rate-limit"] }

# For enterprise with LDAP
poem_auth = { version = "0.1", features = ["sqlite", "ldap", "cache"] }

# Minimal for embedded scenarios
poem_auth = { version = "0.1", features = ["sqlite"] }

# Maximum compatibility
poem_auth = { version = "0.1", features = ["sqlite", "cache", "ldap", "rate-limit", "cors"] }
```

---

## Common Integration Patterns

### Pattern 1: Inject Auth into Poem App State

```rust
use poem::{State, App};
use std::sync::Arc;

// Define app state with auth components
#[derive(Clone)]
pub struct AuthState {
    pub db: Arc<SqliteUserDb>,
    pub provider: Arc<LocalAuthProvider>,
    pub jwt: Arc<JwtValidator>,
}

// Inject into Poem app
let auth_state = AuthState {
    db: Arc::new(SqliteUserDb::new("auth.db").await?),
    provider: Arc::new(LocalAuthProvider::new(db.as_ref().clone())),
    jwt: Arc::new(JwtValidator::new("secret")?),
};

let app = App::new()
    .with_state(auth_state)
    .route(
        Route::new()
            .at("/login", post(login_with_state))
            .at("/protected", get(protected_with_state))
    );

// Use state in handlers
async fn login_with_state(
    req: Json<LoginRequest>,
    State(auth): State<AuthState>,
) -> Result<Json<LoginResponse>, Json<ErrorResponse>> {
    let claims = auth.provider.authenticate(&req.username, &req.password).await?;
    let token = auth.jwt.encode(&claims)?;
    // ... create response
    Ok(Json(response))
}
```

---

### Pattern 2: Middleware Stack Integration

```rust
use poem::{middleware, Route, App};
use tower_http::cors::CorsLayer;

let app = App::new()
    .with(middleware::NormalizePath::new(poem::middleware::NormalizePath::default()))
    .with(middleware::AddData::new(auth_state))
    .with(CorsLayer::very_permissive())
    .route(
        Route::new()
            .at("/login", post(login_handler))
            .at("/protected", get(protected_handler))
    );
```

---

### Pattern 3: Multi-Provider Setup

```rust
use poem_auth::providers::{LocalAuthProvider, LdapAuthProvider};

#[derive(Clone)]
pub struct MultiAuthState {
    local: Arc<LocalAuthProvider>,
    ldap: Option<Arc<LdapAuthProvider>>,
    jwt: Arc<JwtValidator>,
}

async fn authenticate(
    username: &str,
    password: &str,
    provider: &str,
    auth: &MultiAuthState,
) -> Result<UserClaims, AuthError> {
    match provider {
        "local" => auth.local.authenticate(username, password).await,
        "ldap" => {
            if let Some(ldap) = &auth.ldap {
                ldap.authenticate(username, password).await
            } else {
                Err(AuthError::config("LDAP not configured"))
            }
        }
        _ => Err(AuthError::config("Unknown provider")),
    }
}
```

---

### Pattern 4: Custom User Database

Implement custom database backend:

```rust
use async_trait::async_trait;
use poem_auth::{UserDatabase, UserRecord, error::AuthError};

pub struct PostgresUserDb {
    pool: sqlx::PgPool,
}

#[async_trait]
impl UserDatabase for PostgresUserDb {
    async fn get_user(&self, username: &str) -> Result<UserRecord, AuthError> {
        // Implement using your database
        sqlx::query_as::<_, UserRecord>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AuthError::UserNotFound)
    }

    async fn create_user(&self, user: UserRecord) -> Result<(), AuthError> {
        // Implement user creation
        sqlx::query(
            "INSERT INTO users (username, password_hash, groups, enabled) VALUES ($1, $2, $3, $4)"
        )
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(serde_json::to_string(&user.groups).unwrap())
        .bind(user.enabled)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ... implement other trait methods
}
```

---

### Pattern 5: Guarded Routes

```rust
use poem::{http::StatusCode, error::NotFound};

async fn admin_handler(claims: UserClaims) -> Result<String, StatusCode> {
    if !claims.has_group("admins") {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok("Admin access granted".to_string())
}

async fn moderator_or_admin(claims: UserClaims) -> Result<String, StatusCode> {
    if !claims.has_any_group(&["admins", "moderators"]) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok("Moderator access granted".to_string())
}

// Composite guard
async fn sensitive_operation(claims: UserClaims) -> Result<String, StatusCode> {
    if !claims.has_all_groups(&["verified", "admin"]) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok("Operation allowed".to_string())
}
```

---

## Configuration

### Environment-Based Configuration

```rust
use std::env;

pub struct AuthConfig {
    pub db_path: String,
    pub jwt_secret: String,
    pub token_expiry_hours: i64,
    pub master_password: String,
}

impl AuthConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(AuthConfig {
            db_path: env::var("AUTH_DB_PATH").unwrap_or_else(|_| "auth.db".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET environment variable required"),
            token_expiry_hours: env::var("TOKEN_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()?,
            master_password: env::var("MASTER_PASSWORD")
                .expect("MASTER_PASSWORD environment variable required"),
        })
    }
}

// Usage
let config = AuthConfig::from_env()?;
let db = SqliteUserDb::new(&config.db_path).await?;
let jwt = JwtValidator::new(&config.jwt_secret)?;
```

### Configuration File

```rust
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthConfigFile {
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
    pub auto_migrate: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiry_hours: i64,
    pub algorithm: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
}

impl AuthConfigFile {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }
}

// config.toml
// [database]
// path = "auth.db"
// auto_migrate = true
//
// [jwt]
// secret = "your-secret-key-minimum-16-characters"
// expiry_hours = 24
// algorithm = "HS256"
//
// [rate_limit]
// enabled = true
// requests_per_minute = 100
```

---

## Error Handling

### Handling Authentication Errors

```rust
use poem_auth::error::AuthError;
use poem::{error::ResponseError, http::StatusCode};

impl ResponseError for AuthError {
    fn status(&self) -> StatusCode {
        match self {
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthError::UserNotFound => StatusCode::UNAUTHORIZED,
            AuthError::UserDisabled => StatusCode::FORBIDDEN,
            AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
            AuthError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AuthError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// In handlers
async fn protected_endpoint(claims: UserClaims) -> Result<String, AuthError> {
    // Automatically returns appropriate HTTP status
    Ok(format!("Hello {}", claims.sub))
}
```

### Custom Error Response

```rust
use poem::{Response, Body};
use serde_json::json;

async fn login_handler(
    req: Json<LoginRequest>,
    auth: &AuthState,
) -> Result<Response, Response> {
    match auth.provider.authenticate(&req.username, &req.password).await {
        Ok(claims) => {
            let token = auth.jwt.encode(&claims)
                .map_err(|e| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from_json(json!({
                            "error": "token_generation_failed",
                            "message": "Failed to generate authentication token"
                        }))?)
                        .build()
                })?;

            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from_json(json!({
                    "token": token,
                    "token_type": "Bearer"
                }))?)
                .build())
        }
        Err(AuthError::InvalidCredentials) => {
            Err(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from_json(json!({
                    "error": "invalid_credentials",
                    "message": "Invalid username or password"
                }))?)
                .build())
        }
        Err(AuthError::UserDisabled) => {
            Err(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from_json(json!({
                    "error": "user_disabled",
                    "message": "This account has been disabled"
                }))?)
                .build())
        }
        Err(e) => {
            Err(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from_json(json!({
                    "error": "authentication_error",
                    "message": format!("Authentication failed: {}", e)
                }))?)
                .build())
        }
    }
}
```

---

## Testing

### Unit Tests with poem_auth

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use poem_auth::prelude::*;
    use poem_auth::db::sqlite::SqliteUserDb;
    use poem_auth::providers::LocalAuthProvider;
    use poem_auth::password::hash_password;

    #[tokio::test]
    async fn test_authentication_flow() {
        // Setup
        let db = SqliteUserDb::new(":memory:").await.unwrap();
        let hash = hash_password("password123").unwrap();
        let user = UserRecord::new("alice", &hash);
        db.create_user(user).await.unwrap();

        let provider = LocalAuthProvider::new(db);

        // Test
        let result = provider.authenticate("alice", "password123").await;

        // Assert
        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, "alice");
    }

    #[tokio::test]
    async fn test_failed_authentication() {
        let db = SqliteUserDb::new(":memory:").await.unwrap();
        let hash = hash_password("password123").unwrap();
        let user = UserRecord::new("alice", &hash);
        db.create_user(user).await.unwrap();

        let provider = LocalAuthProvider::new(db);

        let result = provider.authenticate("alice", "wrong_password").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_jwt_token_lifecycle() {
        let jwt = JwtValidator::new("test-secret").unwrap();
        let now = chrono::Utc::now().timestamp();
        let claims = UserClaims::new("alice", "local", now + 3600, now);

        // Encode
        let token = jwt.encode(&claims).unwrap();

        // Decode
        let decoded = jwt.decode(&token).unwrap();

        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.provider, claims.provider);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use poem::test::TestClient;
    use poem::{App, Route, post};

    #[tokio::test]
    async fn test_login_endpoint() {
        let auth_state = setup_auth().await;

        let app = App::new()
            .with_state(auth_state)
            .route(Route::new().at("/login", post(login_handler)));

        let client = TestClient::new(app);

        let resp = client
            .post("/login")
            .json(&LoginRequest {
                username: "alice".to_string(),
                password: "password123".to_string(),
                provider: None,
            })
            .send()
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body: LoginResponse = resp.json().await;
        assert!(!body.token.is_empty());
    }
}
```

---

## Production Deployment

### Environment Variables Checklist

```bash
# Required
export JWT_SECRET="your-very-secure-random-secret-at-least-32-characters"
export DATABASE_URL="sqlite:///var/lib/myapp/auth.db"

# Optional
export RUST_LOG="poem_auth=debug"
export RATE_LIMIT_ENABLED="true"
export TOKEN_EXPIRY_HOURS="24"
export LDAP_ENABLED="false"
```

### Docker Integration

```dockerfile
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/myapp /usr/local/bin/
ENV RUST_LOG=poem_auth=info
EXPOSE 3000
CMD ["myapp"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
  template:
    metadata:
      labels:
        app: myapp
    spec:
      containers:
      - name: myapp
        image: myapp:latest
        ports:
        - containerPort: 3000
        env:
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: auth-secrets
              key: jwt-secret
        - name: DATABASE_URL
          valueFrom:
            configMapKeyRef:
              name: auth-config
              key: database-url
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 10
```

### Health Check Endpoint

```rust
#[get("/health")]
async fn health_check(State(auth): State<AuthState>) -> Result<String, StatusCode> {
    // Verify database connection
    match auth.db.list_users().await {
        Ok(_) => Ok("OK".to_string()),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}
```

---

## Troubleshooting

### Issue: "path dependencies require version to be specified"

**Problem:**
```toml
poem_auth = { path = "../rust_based_fast_api" }
```

**Solution:**
```toml
poem_auth = { version = "0.1", path = "../rust_based_fast_api" }
```

---

### Issue: "JWT secret too short"

**Problem:**
```rust
let jwt = JwtValidator::new("secret")?;  // Error!
```

**Solution:**
```rust
let jwt = JwtValidator::new("your-secret-key-minimum-16-characters")?;
```

---

### Issue: "UserClaims not found in scope"

**Problem:**
```rust
async fn handler(claims: UserClaims) -> String { }  // Error!
```

**Solution:**
```rust
use poem_auth::prelude::*;  // Add this import

async fn handler(claims: UserClaims) -> String { }  // Now works
```

---

### Issue: "Cannot find userClaims extractor"

**Problem:**
```rust
// UserClaims not being extracted from request
async fn handler() -> String { }  // No claims injected
```

**Solution:**
```rust
// Need to set Authorization header with Bearer token
// And ensure middleware is properly configured

// In your request:
// Authorization: Bearer <your-jwt-token>

// Also ensure Poem is configured with proper extractors
```

---

### Issue: "Database connection pool exhausted"

**Problem:** Too many concurrent database connections

**Solution:**
```rust
// Increase connection pool size
let pool = SqlitePoolOptions::new()
    .max_connections(20)  // Increase from default
    .connect("sqlite:auth.db")
    .await?;
```

---

### Issue: "LDAP connection timeout"

**Problem:** LDAP server not reachable

**Solution:**
```rust
// 1. Verify LDAP server is running
nc -zv ldap.example.com 389

// 2. Check configuration
let config = LdapConfig {
    server_url: "ldap://ldap.example.com:389".to_string(),
    timeout_seconds: 30,  // Increase timeout
    // ... other config
};

// 3. Validate on startup
ldap_provider.validate_config().await?;
```

---

### Issue: "Rate limiting rejecting all requests"

**Problem:** Rate limit too strict

**Solution:**
```rust
let config = RateLimitConfig {
    general_requests_per_second: 1000,  // Increase from 100
    auth_requests_per_second: 50,       // Increase from 5
};
```

---

## Complete Example Project

```rust
// main.rs
use poem::{
    listener::TcpListener, App, Route, get, post,
    web::Json, http::StatusCode, State,
};
use poem_auth::prelude::*;
use poem_auth::db::sqlite::SqliteUserDb;
use poem_auth::providers::LocalAuthProvider;
use poem_auth::jwt::JwtValidator;
use poem_auth::api::types::{LoginRequest, LoginResponse, ErrorResponse};
use std::sync::Arc;

#[derive(Clone)]
struct AuthState {
    db: Arc<SqliteUserDb>,
    provider: Arc<LocalAuthProvider>,
    jwt: Arc<JwtValidator>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    let db = Arc::new(SqliteUserDb::new("auth.db").await?);
    let provider = Arc::new(LocalAuthProvider::new(db.as_ref().clone()));
    let jwt = Arc::new(JwtValidator::new("your-secret-key-minimum-16-chars")?);

    let state = AuthState { db, provider, jwt };

    // Build app
    let app = App::new()
        .with_state(state)
        .route(
            Route::new()
                .at("/login", post(login))
                .at("/protected", get(protected))
                .at("/health", get(health))
        );

    // Run
    println!("Server running on http://127.0.0.1:3000");
    app.run(TcpListener::bind("127.0.0.1:3000")).await?;

    Ok(())
}

async fn login(
    req: Json<LoginRequest>,
    State(auth): State<AuthState>,
) -> Result<Json<LoginResponse>, Json<ErrorResponse>> {
    let claims = auth.provider.authenticate(&req.username, &req.password)
        .await
        .map_err(|_| Json(ErrorResponse::invalid_credentials()))?;

    let token = auth.jwt.encode(&claims)
        .map_err(|_| Json(ErrorResponse::new("error", "Failed to generate token")))?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: claims.exp - claims.iat,
        claims: poem_auth::api::types::UserClaimsResponse::from_claims(claims),
    }))
}

async fn protected(claims: UserClaims) -> String {
    format!("Hello, {}!", claims.sub)
}

async fn health() -> StatusCode {
    StatusCode::OK
}
```

---

## Summary

To use poem_auth in another project:

1. **Add to Cargo.toml**
   ```toml
   poem_auth = { version = "0.1", path = "../rust_based_fast_api" }
   ```

2. **Import from prelude**
   ```rust
   use poem_auth::prelude::*;
   ```

3. **Initialize components**
   ```rust
   let db = SqliteUserDb::new("auth.db").await?;
   let provider = LocalAuthProvider::new(db);
   let jwt = JwtValidator::new("secret")?;
   ```

4. **Add handlers**
   ```rust
   async fn protected(claims: UserClaims) -> String { }
   ```

5. **Integrate with Poem**
   ```rust
   let app = App::new()
       .at("/protected", get(protected));
   ```

See the [README.md](README.md), [GETTING_STARTED.md](GETTING_STARTED.md), and [API_REFERENCE.md](API_REFERENCE.md) for more details.

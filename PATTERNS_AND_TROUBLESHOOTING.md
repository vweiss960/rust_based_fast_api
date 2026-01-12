# Common Patterns & Troubleshooting

This guide covers common usage patterns and solutions to frequent issues.

## Common Patterns

### Pattern 1: Multiple Authentication Methods

Support both local and LDAP authentication:

```rust
use poem_auth::providers::{LocalAuthProvider, LdapAuthProvider};
use poem_auth::error::AuthError;
use poem_auth::UserClaims;

async fn authenticate(
    username: &str,
    password: &str,
    provider_name: &str,
    local: &LocalAuthProvider,
    ldap: &LdapAuthProvider,
) -> Result<UserClaims, AuthError> {
    match provider_name {
        "local" => local.authenticate(username, password).await,
        "ldap" => ldap.authenticate(username, password).await,
        _ => Err(AuthError::config(format!("Unknown provider: {}", provider_name))),
    }
}

// Or with fallback: try LDAP first, then local
async fn authenticate_with_fallback(
    username: &str,
    password: &str,
    local: &LocalAuthProvider,
    ldap: &LdapAuthProvider,
) -> Result<UserClaims, AuthError> {
    match ldap.authenticate(username, password).await {
        Ok(claims) => Ok(claims),
        Err(_) => local.authenticate(username, password).await,
    }
}
```

### Pattern 2: Role-Based Access Control (RBAC)

```rust
use poem::{Route, get};
use poem_auth::UserClaims;

/// Require a specific role
async fn require_role(
    claims: &UserClaims,
    required_role: &str,
) -> Result<(), String> {
    if claims.has_group(required_role) {
        Ok(())
    } else {
        Err(format!("Role '{}' required", required_role))
    }
}

/// Require any of multiple roles
async fn require_any_role(
    claims: &UserClaims,
    roles: &[&str],
) -> Result<(), String> {
    if claims.has_any_group(roles) {
        Ok(())
    } else {
        Err(format!("One of {:?} roles required", roles))
    }
}

#[get("/admin")]
async fn admin_panel(claims: UserClaims) -> Result<String, String> {
    require_role(&claims, "admins").await?;
    Ok("Admin panel".to_string())
}

#[get("/report")]
async fn report(claims: UserClaims) -> Result<String, String> {
    require_any_role(&claims, &["admins", "managers"]).await?;
    Ok("Report data".to_string())
}
```

### Pattern 3: Middleware-Based Authentication

```rust
use poem::{middleware::Next, Request, Response, Middleware};
use poem_auth::middleware::extract_jwt_claims;
use poem_auth::jwt::JwtValidator;
use poem_auth::error::AuthError;
use std::sync::Arc;

struct AuthMiddleware {
    validator: Arc<JwtValidator>,
}

#[poem::async_trait]
impl Middleware for AuthMiddleware {
    async fn process(&self, req: Request, next: Next) -> Result<Response> {
        // Extract and validate token
        match extract_jwt_claims(&req, &self.validator).await {
            Ok(claims) => {
                // Store claims for downstream handlers
                req.extensions_mut().insert(claims);
                next(req).await
            }
            Err(e) => {
                // Return 401 Unauthorized
                Err(poem::error::Unauthorized.into())
            }
        }
    }
}
```

### Pattern 4: Token Refresh

```rust
use poem::{post, web::Json};
use poem_auth::api::types::{LoginResponse, ErrorResponse, UserClaimsResponse};
use poem_auth::{UserClaims, JwtValidator};

#[post("/refresh")]
async fn refresh_token(
    claims: UserClaims,  // From existing valid token
    jwt: &JwtValidator,
) -> Result<Json<LoginResponse>, Json<ErrorResponse>> {
    let now = chrono::Utc::now().timestamp();

    // Create new claims with updated timestamps
    let new_claims = UserClaims::new(
        &claims.sub,
        &claims.provider,
        now + 86400,  // 24 hours from now
        now,
    )
    .with_groups(claims.groups.clone())
    .with_extra(claims.extra.clone());

    let token = jwt.encode(&new_claims)
        .map_err(|_| Json(ErrorResponse::new("error", "Failed to generate token")))?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: 86400,
        claims: UserClaimsResponse::from_claims(new_claims),
    }))
}
```

### Pattern 5: Custom Claims Storage

```rust
use serde_json::json;
use poem_auth::UserClaims;

// Store additional user data in claims
let claims = UserClaims::new("alice", "local", exp, iat)
    .with_extra(json!({
        "department": "Engineering",
        "clearance_level": 5,
        "team": "Backend",
        "email": "alice@example.com",
    }));

// Retrieve custom claims
if let Some(extra) = &claims.extra {
    if let Some(dept) = extra.get("department") {
        println!("Department: {}", dept);
    }

    if let Some(level) = extra.get("clearance_level").and_then(|v| v.as_i64()) {
        if level >= 5 {
            // Allow sensitive operation
        }
    }
}
```

### Pattern 6: User Management Endpoints

```rust
use poem::{post, get, delete, web::Path, web::Json};
use poem_auth::db::{UserDatabase, UserRecord};
use poem_auth::api::types::{CreateUserRequest, UserResponse, ErrorResponse};
use poem_auth::password::hash_password;
use poem_auth::UserClaims;

#[post("/admin/users")]
async fn create_user(
    claims: UserClaims,  // Require authentication
    req: Json<CreateUserRequest>,
    db: &dyn UserDatabase,
) -> Result<Json<UserResponse>, Json<ErrorResponse>> {
    // Check admin permission
    if !claims.has_group("admins") {
        return Err(Json(ErrorResponse::unauthorized()));
    }

    // Validate username
    if req.username.len() < 3 {
        return Err(Json(ErrorResponse::new(
            "invalid_input",
            "Username must be at least 3 characters",
        )));
    }

    // Check if user exists
    if db.user_exists(&req.username).await {
        return Err(Json(ErrorResponse::new(
            "user_exists",
            "User already exists",
        )));
    }

    // Hash password and create user
    let hash = hash_password(&req.password)
        .map_err(|_| Json(ErrorResponse::new("error", "Password hashing failed")))?;

    let user = UserRecord::new(&req.username, &hash)
        .with_groups(req.groups)
        .with_enabled(req.enabled);

    db.create_user(user).await
        .map_err(|_| Json(ErrorResponse::new("error", "Failed to create user")))?;

    Ok(Json(UserResponse {
        username: req.username.clone(),
        enabled: req.enabled,
        groups: req.groups.clone(),
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    }))
}

#[get("/admin/users/<username>")]
async fn get_user(
    claims: UserClaims,
    username: Path<String>,
    db: &dyn UserDatabase,
) -> Result<Json<UserResponse>, Json<ErrorResponse>> {
    if !claims.has_group("admins") {
        return Err(Json(ErrorResponse::unauthorized()));
    }

    let user = db.get_user(&username).await
        .map_err(|_| Json(ErrorResponse::user_not_found(&username)))?;

    Ok(Json(UserResponse {
        username: user.username,
        enabled: user.enabled,
        groups: user.groups,
        created_at: 0,  // Would come from DB
        updated_at: 0,  // Would come from DB
    }))
}

#[delete("/admin/users/<username>")]
async fn delete_user(
    claims: UserClaims,
    username: Path<String>,
    db: &dyn UserDatabase,
) -> Result<poem::http::StatusCode, Json<ErrorResponse>> {
    if !claims.has_group("admins") {
        return Err(Json(ErrorResponse::unauthorized()));
    }

    db.delete_user(&username).await
        .map_err(|_| Json(ErrorResponse::user_not_found(&username)))?;

    Ok(poem::http::StatusCode::NO_CONTENT)
}
```

### Pattern 7: Audit Logging

```rust
use tracing::{info, warn, error};
use poem_auth::UserClaims;

// Log successful authentication
async fn log_auth_success(username: &str, provider: &str) {
    info!(
        user = username,
        provider = provider,
        "Authentication successful"
    );
}

// Log failed authentication
async fn log_auth_failure(username: &str, reason: &str) {
    warn!(
        user = username,
        reason = reason,
        "Authentication failed"
    );
}

// Log unauthorized access attempt
async fn log_unauthorized_access(claims: &UserClaims, resource: &str) {
    warn!(
        user = claims.sub,
        resource = resource,
        "Unauthorized access attempt"
    );
}

// Log sensitive operations
async fn log_sensitive_operation(claims: &UserClaims, operation: &str) {
    info!(
        user = claims.sub,
        operation = operation,
        "Sensitive operation performed"
    );
}
```

---

## Troubleshooting

### Issue: "Database is locked"

**Cause**: Multiple processes trying to write to SQLite simultaneously.

**Solutions**:

```rust
// For single-threaded use, wrap in Arc and clone:
let db = Arc::new(SqliteUserDb::new("users.db").await?);
let db_clone = db.clone();

// For production, consider:
// 1. Using a dedicated database server (PostgreSQL)
// 2. Implementing connection pooling
// 3. Using async-safe database drivers

// Custom implementation with pooling:
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect("sqlite:users.db")
    .await?;
```

### Issue: "Token validation failed"

**Cause**: Using different secrets for encoding and decoding.

**Solution**:

```rust
// Wrong: Using different secrets
let validator1 = JwtValidator::new("secret1")?;
let validator2 = JwtValidator::new("secret2")?;

let token = validator1.encode(&claims)?;
let decoded = validator2.decode(&token)?;  // Will fail!

// Correct: Use same validator or secret everywhere
let validator = Arc::new(JwtValidator::new("my-secret")?);
let token = validator.encode(&claims)?;
let decoded = validator.decode(&token)?;  // Works!
```

### Issue: "User not found after creating"

**Cause**: Database path issues or async timing.

**Solutions**:

```rust
// Ensure database is persisted before use
let db = SqliteUserDb::new("users.db").await?;

// Verify creation succeeded
let result = db.create_user(user).await?;
assert!(result.is_ok());

// Then verify retrieval
let created_user = db.get_user("alice").await?;
println!("User found: {}", created_user.username);

// Check file exists
use std::path::Path;
assert!(Path::new("users.db").exists());
```

### Issue: "LDAP connection timeout"

**Cause**: LDAP server unreachable or network issues.

**Solutions**:

```rust
use poem_auth::providers::{LdapAuthProvider, LdapConfig};

// Test LDAP connectivity before using
let config = LdapConfig {
    server_url: "ldap://ldap.example.com:389".to_string(),
    timeout_seconds: 10,
    // ... other config
};

match LdapAuthProvider::new(config) {
    Ok(provider) => {
        // Validate on startup
        match provider.validate_config().await {
            Ok(()) => println!("LDAP connection OK"),
            Err(e) => println!("LDAP connection failed: {}", e),
        }
    }
    Err(e) => println!("Failed to initialize LDAP: {}", e),
}

// Troubleshooting steps:
// 1. Check server is reachable: nc -zv ldap.example.com 389
// 2. Verify bind credentials
// 3. Check DN template format
// 4. Enable TLS if required
```

### Issue: "Rate limit blocking legitimate requests"

**Cause**: Limits set too aggressively.

**Solution**:

```rust
use poem_auth::middleware::{RateLimit, RateLimitConfig};

let config = RateLimitConfig {
    general_requests_per_second: 1000,  // Increase from 100
    auth_requests_per_second: 50,       // Increase from 5
};

let limiter = RateLimit::new(config);

// Monitor rate limit hits:
// Log when check fails:
match limiter.check(ip).await {
    Ok(()) => {
        // Process normally
    }
    Err(AuthError::RateLimitExceeded) => {
        eprintln!("Rate limit exceeded for IP: {}", ip);
        // Return 429 Too Many Requests
    }
    Err(e) => eprintln!("Check failed: {}", e),
}
```

### Issue: "Argon2 password hashing slow"

**This is expected behavior** - Argon2 is designed to be slow.

```rust
// The default parameters are intentionally slow (for security):
// Memory: 19,456 KiB (~19 MB)
// Time cost: 2 iterations
// Parallelism: 1 thread
// This takes ~1-2 seconds per password

// If performance is critical:
// 1. Hash passwords offline (during account creation)
// 2. Cache the operation results where safe
// 3. Use background jobs for bulk hashing

// Example: Async hashing in background
use tokio::task;

let password = "user_password".to_string();
let hash = task::spawn_blocking(move || {
    poem_auth::password::hash_password(&password)
})
.await??;
```

### Issue: "Claims not being injected into handlers"

**Cause**: Handler signature doesn't match extractor.

**Solution**:

```rust
// Wrong: UserClaims not extracted
#[get("/protected")]
async fn handler() -> String {
    "Won't have claims".to_string()
}

// Correct: UserClaims as parameter
#[get("/protected")]
async fn handler(claims: UserClaims) -> String {
    format!("Hello {}", claims.sub)
}

// Also correct: Using Result
#[get("/protected")]
async fn handler(claims: Result<UserClaims>) -> Result<String> {
    let claims = claims?;
    Ok(format!("Hello {}", claims.sub))
}
```

### Issue: "Custom claims not persisting"

**Cause**: Claims stored in `extra` field but not accessed correctly.

**Solution**:

```rust
use serde_json::json;

// Creating claims with custom data
let claims = UserClaims::new("alice", "local", exp, iat)
    .with_extra(json!({
        "department": "Engineering",
        "level": 5,
    }));

// Correctly accessing custom claims
if let Some(extra) = &claims.extra {
    let dept = extra.get("department")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let level = extra.get("level")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    println!("Department: {}, Level: {}", dept, level);
}

// Wrong: Trying to access as top-level field
// claims.department  // Won't compile!
```

### Issue: "Token expires immediately"

**Cause**: Incorrect timestamp calculation.

**Solution**:

```rust
use chrono::Utc;

// Wrong: Using same timestamp for exp and iat
let now = 1704067200;
let claims = UserClaims::new("alice", "local", now, now);  // Already expired!

// Correct: exp should be in the future
let now = Utc::now().timestamp();
let exp = now + 86400;  // 24 hours from now
let claims = UserClaims::new("alice", "local", exp, now);

// Verify token lifetime
let ttl = claims.time_to_expiry(now);
println!("Token valid for {} seconds", ttl);
assert!(ttl > 0, "Token already expired!");
```

### Issue: "Groups not appearing in token"

**Cause**: Groups not set when creating claims.

**Solution**:

```rust
// Wrong: No groups set
let claims = UserClaims::new("alice", "local", exp, iat);
assert!(claims.groups.is_empty());

// Correct: Set groups
let claims = UserClaims::new("alice", "local", exp, iat)
    .with_groups(vec!["users", "developers"]);
assert!(!claims.groups.is_empty());

// For LDAP, groups are automatically fetched
let claims = ldap_provider.authenticate("alice", "password").await?;
// claims.groups will contain LDAP groups automatically
```

### Issue: "CORS issues with authentication"

**Cause**: CORS headers not configured for auth endpoints.

**Solution**:

```rust
use tower_http::cors::CorsLayer;
use poem::{middleware::CorsMiddleware, App};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cors = CorsLayer::very_permissive();  // Or configure more strictly

    let app = App::new()
        .with(cors)
        .at("/login", post(login_handler))
        .at("/protected", get(protected_handler));

    Ok(())
}

// Correct CORS configuration for authentication:
// Allow-Origin: Your frontend domain
// Allow-Credentials: true (if using cookies)
// Allow-Methods: POST (for login)
// Allow-Headers: Content-Type, Authorization
```

---

## Testing Patterns

### Unit Testing Providers

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authentication() {
        let db = create_test_db().await;
        let provider = LocalAuthProvider::new(db);

        let result = provider.authenticate("alice", "password123").await;
        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub, "alice");
        assert!(claims.has_group("users"));
    }

    #[tokio::test]
    async fn test_authentication_failure() {
        let db = create_test_db().await;
        let provider = LocalAuthProvider::new(db);

        let result = provider.authenticate("alice", "wrong_password").await;
        assert!(result.is_err());
    }

    async fn create_test_db() -> SqliteUserDb {
        let db = SqliteUserDb::new(":memory:").await.unwrap();
        // Create test users
        db
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_login_flow() {
        // Start app
        // Send login request
        // Verify response contains token
        // Use token to access protected endpoint
        // Verify access granted
    }
}
```

---

## Performance Tips

1. **Use token caching** with `cache` feature for high-traffic applications
2. **Hash passwords asynchronously** in background tasks
3. **Connection pooling** for databases
4. **Rate limit sensitive endpoints** aggressively
5. **Cache LDAP group lookups** if LDAP is slow
6. **Use short-lived tokens** with refresh endpoints
7. **Batch user operations** when possible

---

## Security Checklist

- [ ] JWT secret is random and ≥20 characters
- [ ] HTTPS enforced in production
- [ ] Rate limiting enabled on auth endpoints
- [ ] Password minimum complexity enforced
- [ ] Passwords never logged or exposed
- [ ] Audit logging enabled
- [ ] Token expiration configured appropriately
- [ ] CORS configured restrictively
- [ ] SQL injection protection (using SQLx, not string interpolation)
- [ ] LDAP TLS/STARTTLS enabled for AD connections

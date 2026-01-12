# Test Dependencies & Requirements

Complete breakdown of all dependencies, infrastructure, and setup needed to implement the tests recommended in TEST_COVERAGE_GUIDE.md.

## Table of Contents

- [Summary](#summary)
- [Cargo Dependencies](#cargo-dependencies)
- [Infrastructure Requirements](#infrastructure-requirements)
- [Code to Be Written](#code-to-be-written)
- [Mock Objects & Fixtures](#mock-objects--fixtures)
- [Environment Setup](#environment-setup)
- [Feature Flags](#feature-flags)
- [Hardware & Network](#hardware--network-requirements)
- [Implementation Checklist](#implementation-checklist)

---

## Summary

### At a Glance

| Aspect | Details |
|--------|---------|
| **New Crates Required** | 3 dev dependencies (tracing-test, criterion, proptest) |
| **Code Files to Create** | 9 new test files |
| **Mock Objects** | 5-8 custom mock implementations |
| **External Services** | 1 optional (LDAP server) |
| **Databases** | SQLite only (in-memory for tests) |
| **Infrastructure** | None required (all local/in-memory) |
| **Network Operations** | Only HTTP via TestClient (local) |
| **Environment Variables** | 8 optional configuration options |
| **Secrets** | JWT secret, master password (test values only) |

### Current Status

| Category | Existing | Needed | Gap |
|----------|----------|--------|-----|
| TokenCache | 6 tests | 8 tests | -2 (94% complete) |
| MasterAuth | 6 tests | 6 tests | 0 (100% complete) |
| Audit Logging | 0 tests | 4-5 tests | -4-5 (0% complete) |
| Integration Tests | 0 tests | 10-15 tests | -10-15 (0% complete) |
| Middleware | 0 tests | 4-6 tests | -4-6 (0% complete) |
| Custom Providers | 0 tests | 3-4 tests | -3-4 (0% complete) |
| Custom Database | 0 tests | 3-4 tests | -3-4 (0% complete) |
| Token Refresh | 0 tests | 3-4 tests | -3-4 (0% complete) |
| LDAP | 0 tests | 3-4 tests | -3-4 (0% complete) |
| RBAC | 0 tests | 4-6 tests | -4-6 (0% complete) |
| Error Handling | 0 tests | 5-8 tests | -5-8 (0% complete) |
| Concurrency | 0 tests | 4-6 tests | -4-6 (0% complete) |

---

## Cargo Dependencies

### Current Dev Dependencies (Already Included)

```toml
[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.12"
mockall_double = "0.3"
tempfile = "3"
```

### Required New Dev Dependencies

```toml
[dev-dependencies]
# Logging capture for audit log tests
tracing-test = "0.2"

# Performance benchmarking (optional but recommended)
criterion = "0.5"

# Property-based testing for edge cases (optional but useful)
proptest = "1.0"
```

### Why Each Dependency

| Crate | Version | Purpose | Tests Using It |
|-------|---------|---------|-----------------|
| **tracing-test** | 0.2 | Capture tracing logs in tests | Audit logging (4-5) |
| **criterion** | 0.5 | Benchmarking tool | Concurrency/performance (4-6) |
| **proptest** | 1.0 | Property-based testing | Error handling edge cases (2-3) |

### Dependency Features Required

```bash
# Ensure these features are enabled for testing
cargo test --all-features
cargo test --features "sqlite,cache,ldap,rate-limit,cors"
cargo test --no-default-features --features "sqlite"  # Minimal
```

---

## Infrastructure Requirements

### 1. Database Infrastructure

#### SQLite (Local Only)

| Component | Details | Purpose |
|-----------|---------|---------|
| **Connection Type** | Local file or in-memory | User storage, audit logs |
| **Path** | `:memory:` for tests | Isolated, fast, auto-cleanup |
| **Connection Pool** | sqlx::SqlitePool (5 max) | Concurrent access |
| **Schema Auto-Creation** | sqlx auto-migrates | tables: users, audit_log |

#### Required Schema

```sql
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    groups TEXT NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    username TEXT,
    provider TEXT NOT NULL,
    ip_address TEXT,
    details TEXT
);
```

#### Test Database Setup

```rust
// In test setup
let db = SqliteUserDb::new(":memory:").await?;  // Per-test isolation
```

---

### 2. Optional LDAP/Active Directory Server

#### When Needed

Only for LDAP integration tests (marked with `#[ignore]`):
```rust
#[tokio::test]
#[ignore]  // Run with: cargo test -- --ignored
async fn test_ldap_authentication_with_real_server() { }
```

#### Configuration if Using Real LDAP

```
Server: ldap://dc.example.com:389
Base DN: DC=example,DC=com
Bind DN Template: CN={username},CN=Users,DC=example,DC=com
Group Filter: (member={user_dn})
TLS/STARTTLS: Optional
Timeout: 10 seconds
```

#### For Testing Without Real LDAP

Use mock objects:
```rust
// Mock LDAP DN formatting
fn mock_ldap_bind_dn(username: &str) -> String {
    format!("CN={},CN=Users,DC=example,DC=com", username)
}

// Mock LDAP group lookup
fn mock_ldap_groups(user_dn: &str) -> Vec<String> {
    vec!["users".to_string(), "developers".to_string()]
}
```

---

### 3. In-Memory Infrastructure

#### Token Cache (moka)

```rust
// Already included via cache feature
use moka::future::Cache;

// Usage in tests:
let cache = TokenCache::new(Duration::from_secs(300));
cache.insert(token.clone(), claims.clone()).await;
let cached = cache.get(&token).await;
```

#### Rate Limiting (governor)

```rust
// Already included via rate-limit feature
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

// Usage in tests:
let limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(100).unwrap()));
assert!(limiter.check().is_ok());
```

---

### 4. HTTP Testing Infrastructure

#### Poem Test Client

```rust
use poem::test::TestClient;
use poem::App;

// Create test client for integration tests
let app = App::new()
    .at("/login", post(login_handler))
    .at("/protected", get(protected_handler));

let client = TestClient::new(app);

// Make requests
let resp = client
    .post("/login")
    .json(&LoginRequest { username: "alice", password: "pass" })
    .send()
    .await;

assert_eq!(resp.status(), StatusCode::OK);
```

#### No External Network Required

- All HTTP requests are in-process
- No real network stack involved
- Responses available immediately

---

### 5. Async Runtime

#### Tokio Full Features (Already Configured)

```toml
[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

#### Test Execution Runtime

```rust
#[tokio::test]  // Automatically runs with tokio runtime
async fn my_test() {
    // Async test code
}
```

---

## Code to Be Written

### 1. TokenCache Tests (2 Missing Tests)

**File**: `src/jwt/cache.rs`

**Status**: 6/8 complete - Need 2 more

**Missing Tests**:
```rust
#[tokio::test]
async fn test_cache_expiration() {
    // Verify tokens expire after TTL
    // Check that expired entries return None
}

#[tokio::test]
async fn test_cache_concurrent_access() {
    // Spawn multiple tasks inserting/getting/removing
    // Verify no race conditions
    // Check LRU eviction under load
}
```

---

### 2. Audit Logging Module & Tests

**File**: `src/audit.rs` (NEW) + `tests/audit_logging_tests.rs` (NEW)

**Code to Write**:

```rust
// src/audit.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: i64,
    pub event_type: AuditEventType,
    pub username: Option<String>,
    pub provider: String,
    pub ip_address: Option<String>,
    pub details: Option<String>,
    pub result: AuthResult,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuditEventType {
    LoginSuccess,
    LoginFailed,
    UnauthorizedAccess,
    SensitiveOperation,
    UserCreated,
    UserDeleted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuthResult {
    Success,
    Failure,
}

pub async fn audit_log(event: AuditEvent) {
    tracing::info!(
        event_type = ?event.event_type,
        username = ?event.username,
        provider = %event.provider,
        ip = ?event.ip_address,
        result = ?event.result,
        "Authentication event"
    );
}

// tests/audit_logging_tests.rs
#[test]
#[traced_test]
fn test_successful_login_logged() {
    // Trigger successful login
    // Verify log contains correct fields
}

#[test]
#[traced_test]
fn test_failed_login_logged() {
    // Trigger failed login
    // Verify log contains reason
}
```

---

### 3. Integration Tests

**File**: `tests/integration_tests.rs` (NEW)

**Code to Write**:

```rust
use poem::test::TestClient;
use poem::{App, get, post};
use poem_auth::prelude::*;

async fn setup_test_app() -> TestClient {
    let db = SqliteUserDb::new(":memory:").await.unwrap();

    // Create test user
    let hash = hash_password("password123").unwrap();
    let user = UserRecord::new("alice", &hash)
        .with_groups(vec!["users"]);
    db.create_user(user).await.unwrap();

    let provider = Arc::new(LocalAuthProvider::new(db));
    let jwt = Arc::new(JwtValidator::new("test-secret-min-16-chars").unwrap());

    let app = App::new()
        .at("/login", post({
            let p = provider.clone();
            let j = jwt.clone();
            move |req| login_handler(req, p.clone(), j.clone())
        }))
        .at("/protected", get(protected_handler));

    TestClient::new(app)
}

#[tokio::test]
async fn test_login_flow_end_to_end() {
    let client = setup_test_app().await;

    // 1. Login
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

    // 2. Use token to access protected route
    let resp = client
        .get("/protected")
        .header("Authorization", format!("Bearer {}", body.token))
        .send()
        .await;

    assert_eq!(resp.status(), StatusCode::OK);
}
```

---

### 4. Middleware Integration Tests

**File**: `tests/middleware_tests.rs` (NEW)

**Code to Write**:

```rust
#[tokio::test]
async fn test_rate_limit_middleware() {
    let config = RateLimitConfig {
        general_requests_per_second: 5,
        auth_requests_per_second: 1,
    };
    let limiter = RateLimit::new(config);

    // Should allow first 5 requests
    for i in 0..5 {
        assert!(limiter.check("127.0.0.1").await.is_ok());
    }

    // 6th request should fail
    assert!(matches!(
        limiter.check("127.0.0.1").await,
        Err(AuthError::RateLimitExceeded)
    ));
}

#[tokio::test]
async fn test_jwt_extraction_middleware() {
    let jwt = JwtValidator::new("test-secret").unwrap();
    let claims = UserClaims::new("alice", "local", now + 86400, now);
    let token = jwt.encode(&claims).unwrap();

    let req = Request::builder()
        .header("Authorization", format!("Bearer {}", token))
        .build();

    let extracted = extract_jwt_claims(&req, &jwt).await;
    assert!(extracted.is_ok());
}
```

---

### 5. Custom Provider Tests

**File**: `tests/custom_provider_tests.rs` (NEW)

**Code to Write**:

```rust
struct TestCustomProvider;

#[async_trait]
impl AuthProvider for TestCustomProvider {
    async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<UserClaims, AuthError> {
        if password == "valid" {
            let now = Utc::now().timestamp();
            Ok(UserClaims::new(username, "test", now + 3600, now))
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }

    fn name(&self) -> &str {
        "test_provider"
    }

    async fn validate_config(&self) -> Result<(), AuthError> {
        Ok(())
    }

    fn info(&self) -> String {
        "Test provider".to_string()
    }
}

#[tokio::test]
async fn test_custom_provider_implementation() {
    let provider = TestCustomProvider;
    assert_eq!(provider.name(), "test_provider");

    let result = provider.authenticate("alice", "valid").await;
    assert!(result.is_ok());
    let claims = result.unwrap();
    assert_eq!(claims.sub, "alice");
}
```

---

### 6. Custom Database Tests

**File**: `tests/custom_database_tests.rs` (NEW)

**Code to Write**:

```rust
use dashmap::DashMap;

struct TestDatabase {
    users: DashMap<String, UserRecord>,
}

#[async_trait]
impl UserDatabase for TestDatabase {
    async fn get_user(&self, username: &str) -> Result<UserRecord, AuthError> {
        self.users
            .get(username)
            .map(|u| u.clone())
            .ok_or(AuthError::UserNotFound)
    }

    async fn create_user(&self, user: UserRecord) -> Result<(), AuthError> {
        if self.users.contains_key(&user.username) {
            return Err(AuthError::config("User already exists"));
        }
        self.users.insert(user.username.clone(), user);
        Ok(())
    }

    // ... implement other trait methods
}

#[tokio::test]
async fn test_custom_database_with_provider() {
    let db = TestDatabase {
        users: DashMap::new(),
    };

    let user = UserRecord::new("alice", "hash");
    db.create_user(user).await.unwrap();

    let provider = LocalAuthProvider::new(db);
    let result = provider.authenticate("alice", "password").await;
    // Assert on result
}
```

---

### 7. Token Refresh Tests

**File**: `tests/token_refresh_tests.rs` (NEW)

**Code to Write**:

```rust
#[tokio::test]
async fn test_token_refresh_generates_new_token() {
    let jwt = JwtValidator::new("test-secret").unwrap();
    let old_claims = UserClaims::new("alice", "local", now + 100, now);
    let old_token = jwt.encode(&old_claims).unwrap();

    let old_decoded = jwt.decode(&old_token).unwrap();
    let now = Utc::now().timestamp();
    let new_claims = UserClaims::new(
        &old_decoded.sub,
        &old_decoded.provider,
        now + 86400,
        now,
    )
    .with_groups(old_decoded.groups);

    let new_token = jwt.encode(&new_claims).unwrap();

    // Verify new token is different
    assert_ne!(old_token, new_token);

    // Verify new token has updated timestamps
    let new_decoded = jwt.decode(&new_token).unwrap();
    assert!(new_decoded.iat > old_decoded.iat);
}
```

---

### 8. RBAC Tests

**File**: `tests/rbac_tests.rs` (NEW)

**Code to Write**:

```rust
#[tokio::test]
async fn test_rbac_single_role_required() {
    let admin_claims = UserClaims::new("alice", "local", exp, iat)
        .with_groups(vec!["admins"]);

    let user_claims = UserClaims::new("bob", "local", exp, iat)
        .with_groups(vec!["users"]);

    assert!(admin_claims.has_group("admins"));
    assert!(!user_claims.has_group("admins"));
}

#[tokio::test]
async fn test_rbac_any_of_multiple_roles() {
    let claims = UserClaims::new("alice", "local", exp, iat)
        .with_groups(vec!["users"]);

    assert!(claims.has_any_group(&["admins", "users"]));
    assert!(!claims.has_any_group(&["admins", "moderators"]));
}

#[tokio::test]
async fn test_rbac_all_of_multiple_roles() {
    let claims = UserClaims::new("alice", "local", exp, iat)
        .with_groups(vec!["users", "developers"]);

    assert!(claims.has_all_groups(&["users", "developers"]));
    assert!(!claims.has_all_groups(&["users", "developers", "admins"]));
}
```

---

### 9. Error Handling Tests

**File**: `tests/error_handling_tests.rs` (NEW)

**Code to Write**:

```rust
#[test]
fn test_error_type_detection() {
    let cred_error = AuthError::InvalidCredentials;
    assert!(cred_error.is_invalid_credentials());

    let token_error = AuthError::InvalidToken;
    assert!(token_error.is_token_error());

    let db_error = AuthError::DatabaseError("Connection failed".to_string());
    assert!(db_error.is_database_error());
}

#[test]
fn test_error_display_messages() {
    let error = AuthError::InvalidCredentials;
    assert!(!format!("{}", error).is_empty());
}

#[tokio::test]
async fn test_rate_limit_error_response() {
    let limiter = RateLimit::new(RateLimitConfig {
        general_requests_per_second: 1,
        auth_requests_per_second: 1,
    });

    limiter.check("127.0.0.1").await.unwrap();
    let err = limiter.check("127.0.0.1").await;

    assert!(matches!(err, Err(AuthError::RateLimitExceeded)));
}
```

---

### 10. Concurrency Tests

**File**: `tests/concurrency_tests.rs` (NEW)

**Code to Write**:

```rust
#[tokio::test]
async fn test_concurrent_authentications() {
    let db = SqliteUserDb::new(":memory:").await.unwrap();
    let hash = hash_password("password").unwrap();
    let user = UserRecord::new("alice", &hash);
    db.create_user(user).await.unwrap();

    let provider = Arc::new(LocalAuthProvider::new(db));

    let mut tasks = vec![];
    for _ in 0..100 {
        let p = provider.clone();
        tasks.push(tokio::spawn(async move {
            p.authenticate("alice", "password").await
        }));
    }

    let results = futures::future::join_all(tasks).await;

    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_password_hashing_performance() {
    let start = std::time::Instant::now();

    for _ in 0..10 {
        let _ = hash_password("test_password").unwrap();
    }

    let duration = start.elapsed();
    println!("10 hashes took: {:?}", duration);

    // Each hash should take <500ms
    assert!(duration.as_secs_f64() < 5.0);
}
```

---

## Mock Objects & Fixtures

### 1. Mock AuthProvider

```rust
#[cfg(test)]
mod mocks {
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        pub TestAuthProvider {}

        #[async_trait]
        impl AuthProvider for TestAuthProvider {
            async fn authenticate(&self, username: &str, password: &str)
                -> Result<UserClaims, AuthError>;
            fn name(&self) -> &str;
            async fn validate_config(&self) -> Result<(), AuthError>;
            fn info(&self) -> String;
        }
    }
}
```

### 2. Mock UserDatabase

```rust
mock! {
    pub TestDatabase {}

    #[async_trait]
    impl UserDatabase for TestDatabase {
        async fn get_user(&self, username: &str)
            -> Result<UserRecord, AuthError>;
        async fn create_user(&self, user: UserRecord)
            -> Result<(), AuthError>;
        async fn update_password(&self, username: &str, hash: &str)
            -> Result<(), AuthError>;
        async fn list_users(&self)
            -> Result<Vec<UserRecord>, AuthError>;
        async fn delete_user(&self, username: &str)
            -> Result<(), AuthError>;
    }
}
```

### 3. Test Fixtures

```rust
pub struct TestFixtures;

impl TestFixtures {
    pub fn user_claims(username: &str) -> UserClaims {
        let now = Utc::now().timestamp();
        UserClaims::new(username, "test", now + 86400, now)
            .with_groups(vec!["users"])
    }

    pub fn admin_claims(username: &str) -> UserClaims {
        let now = Utc::now().timestamp();
        UserClaims::new(username, "test", now + 86400, now)
            .with_groups(vec!["admins", "users"])
    }

    pub fn expired_claims(username: &str) -> UserClaims {
        let now = Utc::now().timestamp();
        UserClaims::new(username, "test", now - 3600, now - 86400)
    }

    pub fn test_user_record() -> UserRecord {
        let hash = hash_password("password123").unwrap();
        UserRecord::new("alice", &hash)
            .with_groups(vec!["users"])
    }
}
```

### 4. Mock LDAP Responses

```rust
#[cfg(test)]
pub struct MockLdapServer {
    pub users: HashMap<String, LdapUser>,
}

#[derive(Clone)]
pub struct LdapUser {
    pub dn: String,
    pub groups: Vec<String>,
}

impl MockLdapServer {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(
            "alice".to_string(),
            LdapUser {
                dn: "CN=alice,CN=Users,DC=example,DC=com".to_string(),
                groups: vec!["users".to_string(), "developers".to_string()],
            },
        );
        Self { users }
    }
}
```

---

## Environment Setup

### Configuration Values (Hardcoded in Tests)

```rust
// JWT Configuration
const TEST_JWT_SECRET: &str = "test-secret-key-minimum-16-chars";
const TEST_JWT_EXPIRY_HOURS: i64 = 24;

// Master Authentication
const TEST_MASTER_PASSWORD: &str = "test-master-password";

// Database
const TEST_DATABASE_PATH: &str = ":memory:";  // In-memory for tests

// Rate Limiting
const TEST_RATE_LIMIT_PER_SECOND: u32 = 100;
const TEST_AUTH_ENDPOINT_LIMIT: u32 = 5;

// Token Cache
const TEST_CACHE_TTL_SECONDS: u64 = 300;

// LDAP (optional)
const TEST_LDAP_SERVER: &str = "ldap://localhost:389";
const TEST_LDAP_BASE_DN: &str = "DC=example,DC=com";
```

### Optional Environment Variables

```bash
# Logging output
RUST_LOG=debug
RUST_BACKTRACE=1

# Test database
TEST_DATABASE_PATH=/tmp/poem_auth_test.db

# LDAP configuration (if testing against real LDAP)
LDAP_SERVER=ldap://dc.example.com:389
LDAP_BASE_DN=DC=example,DC=com
LDAP_BIND_DN_TEMPLATE=CN={username},CN=Users,DC=example,DC=com
LDAP_GROUP_FILTER=(member={user_dn})
```

### Test Database Initialization

```rust
// Automatically done by sqlx + SqliteUserDb
#[tokio::test]
async fn test_database_auto_initialized() {
    let db = SqliteUserDb::new(":memory:").await;
    // Tables automatically created
    assert!(db.pool().acquire().await.is_ok());
}
```

---

## Feature Flags

### Test Invocation by Feature Combination

```bash
# Run all tests with all features
cargo test --all-features

# Run with default features only
cargo test

# Run only SQLite tests
cargo test --no-default-features --features "sqlite"

# Run with cache and rate-limit
cargo test --features "cache,rate-limit"

# Run LDAP tests (if available)
cargo test --all-features

# Run ignored tests (LDAP integration with real server)
cargo test -- --ignored --all-features
```

### Feature-Gated Tests

```rust
#[cfg(feature = "ldap")]
mod ldap_tests {
    #[tokio::test]
    #[ignore]  // Optional real LDAP server
    async fn test_ldap_with_real_server() { }

    #[tokio::test]
    async fn test_ldap_configuration() { }
}

#[cfg(feature = "cache")]
mod cache_tests {
    #[tokio::test]
    async fn test_token_cache() { }
}

#[cfg(feature = "rate-limit")]
mod rate_limit_tests {
    #[tokio::test]
    async fn test_rate_limiting() { }
}
```

---

## Hardware & Network Requirements

### Minimum Hardware

| Resource | Requirement | Justification |
|----------|-------------|---|
| **CPU** | 2+ cores | Parallel test execution, concurrent tests |
| **RAM** | 512 MB minimum | Cache, pool connections, test artifacts |
| **Disk** | 100 MB free | SQLite temp files, test output |
| **CPU Speed** | 2+ GHz | Argon2 hashing performance |

### Network Requirements

| Scenario | Requirement | Details |
|----------|-------------|---------|
| **Basic Tests** | None | No external network needed |
| **HTTP Tests** | Localhost only | All via TestClient (in-process) |
| **LDAP Tests** | Optional | Use mock by default, real server optional |
| **Production** | None for testing | Can test everything locally |

### Performance Expectations

| Operation | Time | Purpose |
|-----------|------|---------|
| **Password Hash** | 300-500ms | Argon2 timing |
| **Token Validation** | <1ms | JWT verification |
| **Database Query** | <10ms | SQLite operation |
| **Cache Lookup** | <100μs | In-memory access |
| **Rate Limit Check** | <100μs | Governor check |

### Network Isolation

```
┌─────────────────────────────────────┐
│  Test Environment (All Local)       │
├─────────────────────────────────────┤
│  ✓ SQLite (in-memory)               │
│  ✓ Token Cache (in-memory)          │
│  ✓ Rate Limiter (in-memory)         │
│  ✓ Poem HTTP (TestClient)           │
│  ✗ No external network calls        │
│  ? LDAP Server (optional, mocked)   │
└─────────────────────────────────────┘
```

---

## Implementation Checklist

### Phase 1: Setup & Dependencies (1 week)

- [ ] Add dev dependencies to Cargo.toml:
  - [ ] tracing-test = "0.2"
  - [ ] criterion = "0.5"
  - [ ] proptest = "1.0"
- [ ] Run `cargo test --all-features` to verify setup
- [ ] Set up test infrastructure:
  - [ ] Test fixtures module
  - [ ] Mock objects
  - [ ] Helper functions
- [ ] Document test setup in CONTRIBUTING.md

### Phase 2: Complete TokenCache (1 week)

- [ ] Add missing TokenCache tests:
  - [ ] test_cache_expiration
  - [ ] test_cache_concurrent_access
  - [ ] test_cache_lru_eviction (if applicable)
- [ ] Run `cargo test cache::` to verify
- [ ] Achieve 100% coverage for TokenCache

### Phase 3: Audit Logging (1 week)

- [ ] Create `src/audit.rs` module:
  - [ ] AuditEvent struct
  - [ ] AuditEventType enum
  - [ ] audit_log() function
  - [ ] Tracing integration
- [ ] Create `tests/audit_logging_tests.rs`:
  - [ ] test_successful_login_logged
  - [ ] test_failed_login_logged
  - [ ] test_unauthorized_access_logged
  - [ ] test_sensitive_operation_logged
  - [ ] test_audit_metadata_included
- [ ] Run tracing capture tests: `cargo test audit --all-features`

### Phase 4: Integration Tests (2 weeks)

- [ ] Create `tests/integration_tests.rs`:
  - [ ] setup_test_app() function
  - [ ] test_login_flow_end_to_end
  - [ ] test_login_invalid_credentials
  - [ ] test_protected_route_without_token
  - [ ] test_protected_route_with_expired_token
  - [ ] test_protected_route_with_valid_token
  - [ ] test_provider_fallback (LDAP → local)
  - [ ] test_master_auth_endpoint
- [ ] Run `cargo test integration --all-features`

### Phase 5: Middleware Tests (1 week)

- [ ] Create `tests/middleware_tests.rs`:
  - [ ] test_rate_limit_enforcement
  - [ ] test_rate_limit_auth_stricter
  - [ ] test_rate_limit_per_ip
  - [ ] test_jwt_extraction
  - [ ] test_master_auth_validation
  - [ ] test_cors_headers
- [ ] Run `cargo test middleware --all-features`

### Phase 6: Extensibility Tests (1 week)

- [ ] Create `tests/custom_provider_tests.rs` (3-4 tests)
- [ ] Create `tests/custom_database_tests.rs` (3-4 tests)
- [ ] Create `tests/token_refresh_tests.rs` (3-4 tests)
- [ ] Run `cargo test custom --all-features`

### Phase 7: Advanced Features (1 week)

- [ ] Create `tests/ldap_integration_tests.rs` (3-4 tests)
- [ ] Create `tests/rbac_tests.rs` (4-6 tests)
- [ ] Create `tests/error_handling_tests.rs` (5-8 tests)
- [ ] Run `cargo test advanced --all-features`

### Phase 8: Performance & Concurrency (1 week)

- [ ] Create `tests/concurrency_tests.rs` (4-6 tests)
- [ ] Set up criterion benchmarks (optional)
- [ ] Run `cargo test concurrency --all-features`
- [ ] Document performance baselines

### Phase 9: Coverage & Documentation (1 week)

- [ ] Run full test suite: `cargo test --all-features`
- [ ] Generate coverage report: `cargo tarpaulin --out Html`
- [ ] Update TEST_COVERAGE_GUIDE.md with results
- [ ] Document any gaps or known limitations

---

## Running Tests

### Quick Start

```bash
# Run all tests
cargo test

# Run with all features
cargo test --all-features

# Run specific test category
cargo test cache::          # Token cache tests
cargo test jwt::            # JWT tests
cargo test audit::          # Audit logging tests
cargo test integration::    # Integration tests
cargo test middleware::     # Middleware tests

# Run with output
cargo test -- --nocapture

# Run ignored tests (LDAP with real server)
cargo test -- --ignored --all-features

# Run with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench
```

### CI/CD Integration

```bash
# Full validation pipeline
cargo test --all-features
cargo test --no-default-features
cargo tarpaulin --out Lcov
cargo clippy -- -D warnings
cargo fmt -- --check
```

---

## Summary: What's Needed to Execute Tests

### Installed Software
- ✓ Rust 1.70+ (already have)
- ✓ Cargo (already have)
- ✓ Tokio runtime (already have)
- ? tarpaulin (optional, for coverage reports)
- ? criterion (optional, for benchmarking)

### External Services
- ✗ None required for basic tests
- ? LDAP server (optional, for real LDAP integration)
- ✗ No cloud services needed
- ✗ No databases needed (SQLite in-memory)

### Files to Create
- 9 new test files (~1000-1500 lines total)
- 1 new module file (audit.rs, ~200 lines)
- Updated Cargo.toml (3 new dev dependencies)

### Infrastructure
- ✗ No servers to set up
- ✗ No databases to initialize
- ✓ All local, in-memory

### Effort Estimate
- Setup: 1 week
- TokenCache: 1 week
- Audit logging: 1 week
- Integration tests: 2 weeks
- Middleware: 1 week
- Advanced: 1 week
- Performance: 1 week
- **Total: 9 weeks to 90%+ coverage**

---

*For detailed test implementations, see TEST_COVERAGE_GUIDE.md*

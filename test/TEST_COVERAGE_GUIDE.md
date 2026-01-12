# Test Coverage Guide

Comprehensive guide to test coverage for poem_auth and recommendations for improvement.

## Current Test Coverage Summary

**Total Tests: 73 across 12 modules**
- **Well Covered**: Core types, JWT, password hashing, SQLite, rate limiting
- **Partially Covered**: Local auth, LDAP, API types
- **Missing**: Token caching, master auth, audit logging, integration tests

### Coverage by Feature

| Feature | Status | Tests | Notes |
|---------|--------|-------|-------|
| UserClaims | ✓ Complete | 9 | All methods tested |
| JwtValidator | ✓ Complete | 11 | Creation, encoding, decoding |
| Password Hashing | ✓ Complete | 7 | Hash, verify, edge cases |
| SQLite Database | ✓ Complete | 11 | CRUD, groups, lifecycle |
| LocalAuthProvider | ✓ Complete | 8 | Auth flow, validation |
| LdapAuthProvider | ✓ Good | 13 | Config, DN formatting |
| API Types | ✓ Good | 8 | Serialization, responses |
| Rate Limiting | ✓ Complete | 14 | Limits, configuration |
| **TokenCache** | ✗ **Missing** | 0 | Needs 6-8 tests |
| **MasterAuth** | ✗ **Missing** | 0 | Needs 4-6 tests |
| **Audit Logging** | ✗ **Missing** | 0 | Needs 4-5 tests |
| **Integration Tests** | ✗ **Missing** | 0 | Needs 10-15 tests |

---

## Recommended Tests

### Priority 1: Critical Missing Tests

#### 1. Token Caching Tests (6-8 tests)

**File**: `src/jwt/cache.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        // Test inserting and retrieving a token
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        // Test that cached items expire after TTL
    }

    #[tokio::test]
    async fn test_cache_miss() {
        // Test that missing tokens return None
    }

    #[tokio::test]
    async fn test_cache_clear() {
        // Test clearing all cached entries
    }

    #[tokio::test]
    async fn test_cache_concurrent_access() {
        // Test multiple concurrent reads/writes
    }

    #[tokio::test]
    async fn test_cache_size_limit() {
        // Test that cache respects size limits
    }

    #[tokio::test]
    async fn test_cache_lru_eviction() {
        // Test least-recently-used eviction
    }
}
```

**Test Scenarios**:
- Insert token, retrieve successfully
- Insert token with short TTL, verify expiration
- Retrieve non-existent token (returns None)
- Clear cache and verify all entries gone
- Concurrent inserts and retrievals
- Cache size limits and eviction policy
- LRU behavior under load

---

#### 2. Master Authentication Tests (4-6 tests)

**File**: `src/middleware/master_auth.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use poem::test::TestClient;
    use poem::{Route, post, http::StatusCode};

    #[tokio::test]
    async fn test_master_auth_valid_credentials() {
        // Test successful master auth
    }

    #[tokio::test]
    async fn test_master_auth_invalid_credentials() {
        // Test master auth rejection with wrong password
    }

    #[tokio::test]
    async fn test_master_auth_missing_credentials() {
        // Test master auth when Authorization header is missing
    }

    #[tokio::test]
    async fn test_master_auth_with_bearer_token() {
        // Test extracting password from Bearer token
    }

    #[tokio::test]
    async fn test_master_credentials_constant_time_comparison() {
        // Test timing-safe password comparison
    }

    #[tokio::test]
    async fn test_master_auth_empty_password() {
        // Test master auth with empty password
    }
}
```

**Test Scenarios**:
- Valid master password accepted
- Invalid password rejected
- Missing Authorization header rejected
- Bearer token format handling
- Constant-time comparison (no timing leaks)
- Empty or malformed passwords

---

#### 3. Audit Logging Tests (4-5 tests)

**File**: `src/audit.rs` (create if doesn't exist) or add to relevant modules

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_successful_login_audit_log() {
        // Verify successful login is logged
    }

    #[test]
    #[traced_test]
    fn test_failed_login_audit_log() {
        // Verify failed login attempts are logged
    }

    #[test]
    #[traced_test]
    fn test_unauthorized_access_audit_log() {
        // Verify unauthorized access attempts are logged
    }

    #[test]
    #[traced_test]
    fn test_sensitive_operation_audit_log() {
        // Verify sensitive operations are logged
    }

    #[tokio::test]
    async fn test_audit_log_contains_metadata() {
        // Verify logs contain username, IP, timestamp, etc.
    }
}
```

**Test Scenarios**:
- Successful authentication logged
- Failed authentication logged with reason
- Unauthorized access attempts logged
- Sensitive operations logged
- Audit logs contain proper metadata (user, IP, timestamp)

---

### Priority 2: Integration Tests

#### 4. Complete Authentication Flow Tests (5-8 tests)

**File**: `tests/integration_tests.rs` (new file)

```rust
#[cfg(test)]
mod tests {
    use poem::{test::TestClient, App, listener::TcpListener, post, get};
    use poem_auth::prelude::*;
    use poem_auth::db::sqlite::SqliteUserDb;
    use poem_auth::providers::LocalAuthProvider;
    use poem_auth::jwt::JwtValidator;

    async fn setup_test_app() -> TestClient {
        // Setup app with auth
    }

    #[tokio::test]
    async fn test_login_flow_end_to_end() {
        // Login -> get token -> use token -> verify access
    }

    #[tokio::test]
    async fn test_login_flow_invalid_credentials() {
        // Attempt login with wrong password
    }

    #[tokio::test]
    async fn test_protected_route_without_token() {
        // Access protected route without token -> 401
    }

    #[tokio::test]
    async fn test_protected_route_with_expired_token() {
        // Access protected route with expired token -> 401
    }

    #[tokio::test]
    async fn test_protected_route_with_valid_token() {
        // Access protected route with valid token -> 200
    }

    #[tokio::test]
    async fn test_rbac_authorization() {
        // User with role accesses resource -> 200
        // User without role accesses resource -> 403
    }

    #[tokio::test]
    async fn test_master_auth_endpoint() {
        // Access master auth endpoint
    }

    #[tokio::test]
    async fn test_multiple_providers_fallback() {
        // LDAP fails -> fallback to local auth
    }
}
```

**Test Scenarios**:
- Complete login flow: create user, login, get token, access protected route
- Login fails with invalid credentials
- Accessing protected route without token returns 401
- Accessing protected route with expired token returns 401
- Accessing protected route with valid token returns 200
- RBAC: authorized user can access, unauthorized user cannot
- Master auth endpoints work correctly
- Provider fallback works (try LDAP, fall back to local)

---

#### 5. Middleware Integration Tests (4-6 tests)

**File**: `tests/middleware_tests.rs` (new file)

```rust
#[cfg(test)]
mod tests {
    use poem::{test::TestClient, App, post, get, middleware::AddData};
    use poem_auth::middleware::{RateLimit, RateLimitConfig};

    #[tokio::test]
    async fn test_rate_limit_middleware() {
        // Requests under limit succeed
        // Requests over limit fail with 429
    }

    #[tokio::test]
    async fn test_rate_limit_auth_endpoint_stricter() {
        // Auth endpoints have stricter limits
    }

    #[tokio::test]
    async fn test_rate_limit_ip_tracking() {
        // Different IPs tracked separately
    }

    #[tokio::test]
    async fn test_jwt_extraction_middleware() {
        // Valid token extracted
        // Invalid token rejected
        // Missing token rejected
    }

    #[tokio::test]
    async fn test_master_auth_middleware() {
        // Valid master password succeeds
        // Invalid password fails
    }

    #[tokio::test]
    async fn test_cors_with_auth_endpoints() {
        // CORS headers present on auth endpoints
    }
}
```

**Test Scenarios**:
- Rate limiting middleware enforces limits
- Auth endpoints have stricter rate limits
- Different IPs tracked separately
- JWT extraction works with valid/invalid tokens
- Master auth middleware validation
- CORS headers on authentication endpoints

---

### Priority 3: Feature-Specific Tests

#### 6. Custom Provider Implementation Tests (3-4 tests)

**File**: `tests/custom_provider_tests.rs` (new file)

```rust
#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use poem_auth::{AuthProvider, UserClaims, error::AuthError};

    struct TestCustomProvider;

    #[async_trait]
    impl AuthProvider for TestCustomProvider {
        async fn authenticate(
            &self,
            username: &str,
            password: &str,
        ) -> Result<UserClaims, AuthError> {
            // Test implementation
            todo!()
        }

        fn name(&self) -> &str {
            "test"
        }

        async fn validate_config(&self) -> Result<(), AuthError> {
            Ok(())
        }

        fn info(&self) -> String {
            "Test provider".to_string()
        }
    }

    #[tokio::test]
    async fn test_custom_provider_trait_implementation() {
        // Custom provider works with auth system
    }

    #[tokio::test]
    async fn test_custom_provider_with_claims() {
        // Custom provider can set groups and extra claims
    }

    #[tokio::test]
    async fn test_custom_provider_error_handling() {
        // Custom provider error propagation
    }

    #[tokio::test]
    async fn test_custom_provider_validation() {
        // Provider config validation called on init
    }
}
```

**Test Scenarios**:
- Custom provider authentication works
- Custom provider sets groups and claims correctly
- Errors from custom providers propagate
- Config validation is called

---

#### 7. Custom Database Implementation Tests (3-4 tests)

**File**: `tests/custom_database_tests.rs` (new file)

```rust
#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use poem_auth::{UserDatabase, UserRecord, error::AuthError};

    struct TestDatabase;

    #[async_trait]
    impl UserDatabase for TestDatabase {
        async fn get_user(&self, username: &str) -> Result<UserRecord, AuthError> {
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

    #[tokio::test]
    async fn test_custom_database_implementation() {
        // Custom database works with auth system
    }

    #[tokio::test]
    async fn test_custom_database_with_local_provider() {
        // LocalAuthProvider works with custom database
    }

    #[tokio::test]
    async fn test_custom_database_group_management() {
        // Custom database can manage user groups
    }

    #[tokio::test]
    async fn test_custom_database_password_updates() {
        // Password updates work correctly
    }
}
```

**Test Scenarios**:
- Custom database implementation works
- LocalAuthProvider works with custom database
- Group management through custom database
- Password updates through custom database

---

#### 8. Token Refresh Flow Tests (3-4 tests)

**File**: `tests/token_refresh_tests.rs` (new file)

```rust
#[cfg(test)]
mod tests {
    use poem::{test::TestClient, post, web::Json};
    use poem_auth::UserClaims;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_token_refresh_generates_new_token() {
        // Old token + refresh endpoint -> new token
    }

    #[tokio::test]
    async fn test_token_refresh_updates_timestamps() {
        // New token has updated iat and exp
    }

    #[tokio::test]
    async fn test_token_refresh_preserves_claims() {
        // Groups and extra claims preserved
    }

    #[tokio::test]
    async fn test_token_refresh_expired_token_rejected() {
        // Expired token cannot be refreshed
    }
}
```

**Test Scenarios**:
- Token refresh generates new token with new timestamps
- Groups and custom claims preserved across refresh
- Expired tokens cannot be refreshed
- Token refresh extends session

---

#### 9. LDAP Integration Tests (3-4 tests)

**File**: `tests/ldap_integration_tests.rs` (new file)

```rust
#[cfg(test)]
#[cfg(feature = "ldap")]
mod tests {
    use poem_auth::providers::{LdapAuthProvider, LdapConfig};

    #[tokio::test]
    #[ignore]  // Requires LDAP server
    async fn test_ldap_authentication_with_real_server() {
        // Connect to real LDAP server
    }

    #[tokio::test]
    async fn test_ldap_configuration_validation() {
        // Invalid config rejected
    }

    #[tokio::test]
    async fn test_ldap_dn_template_formatting() {
        // DN templates formatted correctly
    }

    #[tokio::test]
    async fn test_ldap_group_retrieval() {
        // User groups retrieved from LDAP
    }
}
```

**Test Scenarios**:
- LDAP server connectivity (when available)
- Configuration validation
- DN template formatting
- Group retrieval from LDAP

---

#### 10. Role-Based Access Control (RBAC) Tests (4-6 tests)

**File**: `tests/rbac_tests.rs` (new file)

```rust
#[cfg(test)]
mod tests {
    use poem::{test::TestClient, get};
    use poem_auth::UserClaims;

    #[tokio::test]
    async fn test_rbac_single_role_required() {
        // User with role succeeds, without role fails
    }

    #[tokio::test]
    async fn test_rbac_any_of_multiple_roles() {
        // has_any_group() works correctly
    }

    #[tokio::test]
    async fn test_rbac_all_of_multiple_roles() {
        // has_all_groups() works correctly
    }

    #[tokio::test]
    async fn test_rbac_nested_role_checks() {
        // Complex authorization logic
    }

    #[tokio::test]
    async fn test_rbac_custom_authorization_middleware() {
        // Custom auth middleware respects roles
    }

    #[tokio::test]
    async fn test_rbac_forbidden_response() {
        // Unauthorized users get 403 Forbidden
    }
}
```

**Test Scenarios**:
- Single role requirement
- Any-of-multiple roles
- All-of-multiple roles
- Complex authorization logic
- Custom middleware respects roles
- 403 response for unauthorized access

---

### Priority 4: Edge Cases and Error Handling

#### 11. Error Handling Tests (5-8 tests)

**File**: `tests/error_handling_tests.rs` (new file)

```rust
#[cfg(test)]
mod tests {
    use poem_auth::error::AuthError;

    #[test]
    fn test_error_display_messages() {
        // Error messages are meaningful
    }

    #[test]
    fn test_error_type_detection() {
        // is_invalid_credentials(), is_token_error(), etc.
    }

    #[tokio::test]
    async fn test_database_error_handling() {
        // Database errors propagate correctly
    }

    #[tokio::test]
    async fn test_jwt_error_handling() {
        // JWT errors propagate correctly
    }

    #[tokio::test]
    async fn test_ldap_error_handling() {
        // LDAP errors handled gracefully
    }

    #[tokio::test]
    async fn test_rate_limit_error_response() {
        // Rate limit errors return 429
    }

    #[tokio::test]
    async fn test_error_response_json_format() {
        // Error responses are valid JSON
    }

    #[tokio::test]
    async fn test_error_details_included() {
        // Error details included when present
    }
}
```

**Test Scenarios**:
- Error messages are meaningful
- Error type detection methods work
- Database errors propagate
- JWT errors propagate
- LDAP errors handled
- Rate limit errors return correct status
- Error response format is valid JSON
- Error details included

---

#### 12. Concurrency and Performance Tests (4-6 tests)

**File**: `tests/concurrency_tests.rs` (new file)

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_concurrent_authentications() {
        // Multiple concurrent auth requests
    }

    #[tokio::test]
    async fn test_concurrent_token_validations() {
        // Multiple concurrent token validations
    }

    #[tokio::test]
    async fn test_concurrent_cache_access() {
        // Cache handles concurrent access
    }

    #[tokio::test]
    async fn test_password_hashing_performance() {
        // Hash operations take reasonable time
    }

    #[tokio::test]
    async fn test_token_validation_performance() {
        // Token validation is fast
    }

    #[tokio::test]
    async fn test_database_concurrent_writes() {
        // Database handles concurrent writes
    }
}
```

**Test Scenarios**:
- Multiple concurrent auth requests handled
- Concurrent token validations work
- Cache handles concurrent access safely
- Password hashing performance acceptable
- Token validation is fast
- Database handles concurrent writes

---

## Implementation Priority

### Phase 1 (2-3 weeks) - Critical Tests
- [ ] Token caching tests (6-8 tests)
- [ ] Master authentication tests (4-6 tests)
- [ ] Audit logging tests (4-5 tests)
- [ ] Basic integration tests (5-8 tests)

**Expected Coverage Increase**: 20-25 tests, covers 3 major missing features

### Phase 2 (1-2 weeks) - Integration Tests
- [ ] Middleware integration tests (4-6 tests)
- [ ] Complete authentication flow tests (5-8 tests)
- [ ] RBAC tests (4-6 tests)

**Expected Coverage Increase**: 13-20 tests, covers end-to-end scenarios

### Phase 3 (1 week) - Extension Tests
- [ ] Custom provider tests (3-4 tests)
- [ ] Custom database tests (3-4 tests)
- [ ] Token refresh tests (3-4 tests)

**Expected Coverage Increase**: 9-12 tests, covers extensibility

### Phase 4 (1 week) - Polish Tests
- [ ] Error handling tests (5-8 tests)
- [ ] Concurrency tests (4-6 tests)
- [ ] Edge cases and performance

**Expected Coverage Increase**: 9-14 tests, covers edge cases

---

## Total Test Estimates

| Phase | New Tests | Total After | Coverage % |
|-------|-----------|------------|-----------|
| Current | - | 73 | ~50% |
| Phase 1 | 19-27 | 92-100 | ~65% |
| Phase 2 | 13-20 | 105-120 | ~80% |
| Phase 3 | 9-12 | 114-132 | ~85% |
| Phase 4 | 9-14 | 123-146 | ~90%+ |

---

## Test Organization Structure

```
tests/
├── integration_tests.rs          # End-to-end flows
├── middleware_tests.rs           # Middleware integration
├── rbac_tests.rs                 # Authorization
├── custom_provider_tests.rs      # Custom implementations
├── custom_database_tests.rs      # Custom DB backends
├── token_refresh_tests.rs        # Token lifecycle
├── ldap_integration_tests.rs     # LDAP-specific (conditional)
├── error_handling_tests.rs       # Error scenarios
└── concurrency_tests.rs          # Performance & concurrency

src/
├── jwt/
│   └── cache.rs                  # Add tests here
├── middleware/
│   └── master_auth.rs            # Add tests here
└── lib.rs                        # Add audit logging tests
```

---

## Running Tests

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Run specific test module
cargo test jwt::

# Run with output
cargo test -- --nocapture

# Run ignored tests (e.g., LDAP integration)
cargo test -- --ignored

# Test with all features
cargo test --all-features

# Test with no default features
cargo test --no-default-features
```

---

## Coverage Tools Setup

Add to `Cargo.toml` (dev dependencies):

```toml
[dev-dependencies]
tarpaulin = "0.21"           # Code coverage
tracing-test = "0.2"         # Tracing in tests
criterion = "0.5"            # Benchmarking
proptest = "1.0"             # Property-based testing
```

```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/

# Generate with timeout
cargo tarpaulin --timeout 300 --out Html
```

---

## Test Quality Guidelines

1. **Each test should be independent** - No shared state
2. **Use descriptive names** - `test_admin_access_denied_for_user_without_role`
3. **One assertion per scenario** - Use `assert_*` macros
4. **Mock external dependencies** - Don't hit real LDAP/databases
5. **Clean up resources** - Temp databases, files, etc.
6. **Document complex tests** - Comments for non-obvious behavior
7. **Test both happy and sad paths** - Success and failure cases
8. **Include integration tests** - Not just unit tests

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Total Tests | 73 | 150+ |
| Line Coverage | 40-50% | 80%+ |
| Feature Coverage | 60% | 100% |
| Integration Tests | 0 | 20+ |
| Middleware Tests | 1 | 10+ |
| Error Cases | 4 | 20+ |
| Concurrency Tests | 0 | 5+ |

# Phase 2 Quick Reference

## What's New

**Automatic User Claims Extraction** + **Composable Authorization Guards**

## Quick Start

### Protected Endpoint (Auto Extraction)

```rust
#[handler]
async fn protected(claims: UserClaims) -> Response {
    // claims automatically extracted from Authorization header!
    (StatusCode::OK, Json(json!({
        "user": claims.sub,
        "groups": claims.groups
    }))).into_response()
}
```

### Admin-Only Endpoint (Guard Check)

```rust
#[handler]
async fn admin_only(claims: UserClaims) -> Response {
    let guard = HasGroup("admins".to_string());

    if guard.check(&claims) {
        // Admin action
        (StatusCode::OK, Json(json!({"access": "granted"}))).into_response()
    } else {
        (StatusCode::FORBIDDEN, Json(json!({"error": "admin required"}))).into_response()
    }
}
```

## Imports

```rust
use poem_auth::{
    UserClaims,
    poem_integration::guards::{AuthGuard, HasGroup, HasAnyGroup, HasAllGroups, And, Or, Not}
};
```

## Guard Types

### Single Group Check
```rust
let guard = HasGroup("admins".to_string());
guard.check(&claims) // true if user has 'admins' group
```

### Any Group Match (OR logic)
```rust
let guard = HasAnyGroup(vec!["admin".to_string(), "moderator".to_string()]);
guard.check(&claims) // true if user has admin OR moderator
```

### All Groups Match (AND logic)
```rust
let guard = HasAllGroups(vec!["developer".to_string(), "verified".to_string()]);
guard.check(&claims) // true if user has BOTH developer AND verified
```

## Composable Guards

### AND Operator
```rust
let guard = And(
    HasGroup("admin".to_string()),
    HasGroup("verified".to_string())
);
// User must be admin AND verified
```

### OR Operator
```rust
let guard = Or(
    HasGroup("admin".to_string()),
    HasGroup("moderator".to_string())
);
// User can be admin OR moderator
```

### NOT Operator
```rust
let guard = Not(HasGroup("banned".to_string()));
// User must NOT be banned
```

### Complex Combinations
```rust
let guard = And(
    Or(
        HasGroup("admin".to_string()),
        HasGroup("moderator".to_string())
    ),
    Not(HasGroup("disabled".to_string()))
);
// Must be (admin OR moderator) AND NOT disabled
```

## Builder Functions

```rust
use poem_auth::poem_integration::guards::builders::*;

let guard1 = require_group("admin");
let guard2 = require_any_group(vec!["admin", "moderator"]);
let guard3 = require_all_groups(vec!["developer", "verified"]);
```

## UserClaims Methods

```rust
let claims = UserClaims::new("alice", "local", exp, iat)
    .with_groups(vec!["users", "admins"]);

// Check membership
claims.has_group("admins")           // true
claims.has_any_group(&["admin", "moderator"]) // true
claims.has_all_groups(&["users", "admins"])  // true

// Get info
claims.sub                           // "alice"
claims.groups                        // vec!["users", "admins"]
claims.provider                      // "local"
claims.exp                           // expiration timestamp
claims.iat                           // issued at timestamp
```

## FromRequest Behavior

When a handler has a `UserClaims` parameter:

1. Poem automatically calls `FromRequest::from_request()`
2. Your custom impl:
   - Extracts Authorization header
   - Validates Bearer token format
   - Verifies JWT signature
   - Returns claims or 401 Unauthorized

**Example Request**:
```bash
curl -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGc..." http://localhost:3000/protected
```

**No Authorization Header** → 401 Unauthorized
**Invalid Bearer Format** → 401 Unauthorized
**Expired Token** → 401 Unauthorized
**Valid Token** → Claims extracted, handler called with claims parameter

## Error Responses

### FromRequest Failure (401)
```json
{
  "error": "Unauthorized"
}
```

### Guard Check Failure (403)
```json
{
  "error": "This endpoint requires 'admins' group membership"
}
```

## Phase 1 + Phase 2 Combined

```rust
use poem_auth::{
    initialize_from_config,           // Phase 1
    PoemAppState,                      // Phase 1
    UserClaims,                        // Phase 2
    poem_integration::guards::*        // Phase 2
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Phase 1: One-line config-driven setup
    initialize_from_config("auth.toml").await?;

    let app = Route::new()
        // Phase 2: Auto-extracting handlers with guards
        .at("/api/user", get(user_profile))
        .at("/api/admin", get(admin_panel))
        .at("/api/mod", get(moderator_console));

    Server::new(TcpListener::bind("0.0.0.0:3000")).run(app).await?;
    Ok(())
}

// Auto extraction - no manual token parsing!
#[handler]
async fn user_profile(claims: UserClaims) -> String {
    format!("User: {}", claims.sub)
}

// Guard-based authorization - minimal code
#[handler]
async fn admin_panel(claims: UserClaims) -> Response {
    if HasGroup("admins".to_string()).check(&claims) {
        (StatusCode::OK, "Admin panel").into_response()
    } else {
        (StatusCode::FORBIDDEN, "Access denied").into_response()
    }
}

// Composable guards - reusable logic
#[handler]
async fn moderator_console(claims: UserClaims) -> Response {
    if HasAnyGroup(vec!["admin".to_string(), "moderator".to_string()]).check(&claims) {
        (StatusCode::OK, "Moderator console").into_response()
    } else {
        (StatusCode::FORBIDDEN, "Moderators only").into_response()
    }
}
```

## What's Still Coming (Phase 2b+)

### Procedural Macros (Phase 2b)
```rust
#[require_groups("admin")]
#[handler]
async fn future_admin_endpoint(claims: UserClaims) -> Response {
    // Guards applied automatically by macro!
    (StatusCode::OK, "Admin area").into_response()
}
```

This will be available in a separate `poem_auth_macros` crate.

## Files Changed

**Core Library**
- `src/poem_integration/extractors.rs` - NEW
- `src/poem_integration/guards.rs` - NEW
- `src/poem_integration/mod.rs` - UPDATED
- `src/lib.rs` - UPDATED (exports Phase 2 types)

**Examples**
- `examples/poem_example/src/main.rs` - UPDATED with Phase 2 endpoints
- `examples/poem_example/auth.toml` - UPDATED with test users

## Compilation Status

✅ Library builds without errors
✅ Example application builds and runs
✅ All guard types compile and execute
✅ FromRequest trait properly implemented

## Testing Phase 2

```bash
# Start the server
cd examples/poem_example
cargo run

# Get a token
TOKEN=$(curl -X POST http://localhost:3000/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"password123"}' | jq -r '.token')

# Test auto-extraction (protected endpoint)
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/protected

# Test admin guard
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/admin  # Works - alice is admin

# Test with non-admin token
TOKEN2=$(curl -X POST http://localhost:3000/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"bob","password":"secret456"}' | jq -r '.token')

curl -H "Authorization: Bearer $TOKEN2" http://localhost:3000/admin  # 403 Forbidden - bob not admin

# Test moderator endpoint
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/moderator  # Works - admin is also moderator allowed
```

## Performance Notes

- FromRequest extraction is fast (single header lookup)
- Guard checks are inlined by Rust compiler
- Token validation is cached if `cache` feature enabled
- Zero-cost abstractions - no runtime overhead

## Security Notes

- All 401 responses on extraction failure prevent information leakage
- Guard checks are stateless and fast
- UserClaims is immutable in handlers
- Token expiration is checked during verification

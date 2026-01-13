# Phase 2B Quick Reference - Macro-based Authorization

## What's New

Three procedural macros for zero-boilerplate authorization on Poem handlers.

## The Three Macros

### 1. `#[require_group("name")]` - Single Group Check

**Usage**: User must have the specified group

```rust
#[require_group("admins")]
#[handler]
async fn admin_area(claims: UserClaims) -> Response {
    json!({"area": "admin"}).into()
}
```

**Expands to**: Guard check inserted at function start
```rust
let __guard = ::poem_auth::HasGroup("admins".to_string());
if !__guard.check(&claims) {
    return (StatusCode::FORBIDDEN, Json(json!({"error": "..."})))).into_response();
}
```

### 2. `#[require_any_groups("g1", "g2", ...)]` - OR Logic

**Usage**: User must have ANY of the specified groups

```rust
#[require_any_groups("admins", "moderators")]
#[handler]
async fn moderation(claims: UserClaims) -> Response {
    json!({"panel": "moderation"}).into()
}
```

**Behavior**: Access granted if user has admin OR moderator

### 3. `#[require_all_groups("g1", "g2", ...)]` - AND Logic

**Usage**: User must have ALL of the specified groups

```rust
#[require_all_groups("developers", "verified")]
#[handler]
async fn dev_area(claims: UserClaims) -> Response {
    json!({"area": "dev"}).into()
}
```

**Behavior**: Access granted only if user has BOTH developers AND verified

## Imports

```rust
use poem_auth::{
    UserClaims,
    require_group,
    require_any_groups,
    require_all_groups,
};
```

## Error Responses

All macros return 403 Forbidden on failed checks:

```json
{
  "error": "Forbidden: requires 'admins' group"
}
```

Or for multiple groups:

```json
{
  "error": "Forbidden: requires one of groups: admins, moderators"
}
```

## Complete Example

```rust
use poem::{handler, Route, Server, listener::TcpListener, Response, get, post, web::Json, http::StatusCode};
use poem_auth::{
    initialize_from_config, UserClaims,
    require_group, require_any_groups, require_all_groups,
};
use serde_json::json;

// One-liner setup (Phase 1)
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_from_config("auth.toml").await?;

    let app = Route::new()
        .at("/user", get(user_area))
        .at("/admin", get(admin_area))
        .at("/mod", get(mod_area))
        .at("/dev", get(dev_area));

    Server::new(TcpListener::bind("0.0.0.0:3000")).run(app).await?;
    Ok(())
}

// Automatic extraction (Phase 2)
// + Macro protection (Phase 2B)
#[handler]
async fn user_area(claims: UserClaims) -> Response {
    json!({"user": claims.sub}).into()
}

// Single group requirement
#[require_group("admins")]
#[handler]
async fn admin_area(claims: UserClaims) -> Response {
    json!({"area": "admin_dashboard"}).into()
}

// Multiple groups with OR logic
#[require_any_groups("admins", "moderators")]
#[handler]
async fn mod_area(claims: UserClaims) -> Response {
    json!({"area": "moderation"}).into()
}

// Multiple groups with AND logic
#[require_all_groups("developers", "verified")]
#[handler]
async fn dev_area(claims: UserClaims) -> Response {
    json!({"area": "developer_tools"}).into()
}
```

## Macro Stacking

Combine multiple macros for complex authorization:

```rust
// Must be verified AND (admin OR moderator)
#[require_group("verified")]
#[require_any_groups("admins", "moderators")]
#[handler]
async fn special_panel(claims: UserClaims) -> Response {
    json!({"panel": "special"}).into()
}
```

Macros are evaluated from outermost to innermost.

## Requirements

Handlers using authorization macros MUST:

1. **Have claims parameter**: `claims: UserClaims`
   ```rust
   #[require_group("admin")]
   #[handler]
   async fn handler(claims: UserClaims) -> Response { ... }  // ✅ OK
   ```

2. **Return IntoResponse type**: `Response`, `String`, JSON, etc.
   ```rust
   #[require_group("admin")]
   #[handler]
   async fn handler(claims: UserClaims) -> Response { ... }  // ✅ OK
   ```

3. **Be async or sync handler**:
   ```rust
   #[require_group("admin")]
   #[handler]
   async fn async_handler(claims: UserClaims) -> Response { ... }  // ✅ OK

   #[require_group("admin")]
   #[handler]
   fn sync_handler(claims: UserClaims) -> String { ... }  // ✅ OK
   ```

## Comparison: Before vs After

### Before (Manual Guards)

```rust
#[handler]
async fn admin_endpoint(claims: UserClaims) -> Response {
    // Manual guard creation
    let guard = HasGroup("admins".to_string());

    // Manual check
    if guard.check(&claims) {
        // Business logic
        (StatusCode::OK, Json(json!({
            "message": "Admin area"
        }))).into_response()
    } else {
        // Manual error response
        (StatusCode::FORBIDDEN, Json(json!({
            "error": "Forbidden: requires 'admins' group"
        }))).into_response()
    }
}
```

**Total: 15+ lines, authorization mixed with logic**

### After (With Macro)

```rust
#[require_group("admins")]
#[handler]
async fn admin_endpoint(claims: UserClaims) -> Response {
    // Business logic only!
    (StatusCode::OK, Json(json!({
        "message": "Admin area"
    }))).into_response()
}
```

**Total: 5 lines, authorization separated**

## Common Patterns

### Simple Admin Area
```rust
#[require_group("admin")]
#[handler]
async fn admin(claims: UserClaims) -> Response { ... }
```

### Multi-role Access
```rust
#[require_any_groups("admin", "moderator", "support")]
#[handler]
async fn support_tools(claims: UserClaims) -> Response { ... }
```

### Certified Developers
```rust
#[require_all_groups("developer", "certified")]
#[handler]
async fn production_deploy(claims: UserClaims) -> Response { ... }
```

### Layered Security
```rust
#[require_group("verified")]  // Must be verified
#[require_any_groups("admin", "lead")]  // AND (admin OR lead)
#[handler]
async fn sensitive_action(claims: UserClaims) -> Response { ... }
```

## Testing the Macros

### Get a token
```bash
TOKEN=$(curl -X POST http://localhost:3000/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"password123"}' | jq -r '.token')
```

### Access macro-protected endpoint
```bash
# Should work (alice is admin)
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/admin/macro

# Response: 200 OK
# {"message":"Admin access granted via macro!","username":"alice"}
```

### Test with non-admin user
```bash
TOKEN2=$(curl -X POST http://localhost:3000/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"bob","password":"secret456"}' | jq -r '.token')

curl -H "Authorization: Bearer $TOKEN2" http://localhost:3000/admin/macro

# Response: 403 Forbidden
# {"error":"Forbidden: requires 'admins' group"}
```

## How It Works

1. **Compile Time**: Macro processes function and arguments
2. **Parse**: Extracts group names from attribute
3. **Validate**: Checks for `claims: UserClaims` parameter
4. **Generate**: Creates guard check code
5. **Insert**: Adds check at function start before business logic
6. **Return**: Modified function ready for Poem

## Feature Flags

Macros are enabled by default but optional:

```toml
# Disable macros (if not needed)
poem_auth = { version = "0.1", default-features = false, features = ["sqlite"] }

# Macros require poem_auth_macros crate, which is optional
```

## Error Messages

### Missing claims parameter
```
error: Handler must have a `claims: UserClaims` parameter to use authorization macros
```

### No groups specified
```
error: At least one group must be specified
```

### Invalid syntax
```
error: expected string literal
```

## Performance

- **Compile time**: ~10-50ms per macro (negligible)
- **Runtime**: 0ms (inlined by compiler, no overhead)
- **Binary size**: No increase (guards already exist in Phase 2)

## Disabling for a Handler

If you need to access a protected endpoint without macro guard:

```rust
#[handler]
async fn unprotected(claims: UserClaims) -> Response {
    // No macro = no automatic guard check
    // But claims are still extracted and available
    json!({"access": "free"}).into()
}
```

## Accessing Claims in Handler

Macros don't change how you use claims:

```rust
#[require_group("user")]
#[handler]
async fn profile(claims: UserClaims) -> Response {
    json!({
        "username": claims.sub,
        "groups": claims.groups,
        "provider": claims.provider,
        "expires_in": claims.exp - claims.iat
    }).into()
}
```

Full access to UserClaims fields after guard check passes.

## Common Issues

### Macro not found
```
error: cannot find attribute macro `require_group` in this scope
```

**Fix**: Add import
```rust
use poem_auth::require_group;
```

### Missing claims parameter
```
error: Handler must have a `claims: UserClaims` parameter
```

**Fix**: Add claims parameter
```rust
#[require_group("admin")]
#[handler]
async fn handler(claims: UserClaims) -> Response { ... }
```

### Wrong group type
```
error: expected string literal
```

**Fix**: Use string literals, not variables
```rust
#[require_group("admin")]  // ✅
#[require_group(GROUP)]    // ❌
```

## Next Steps

1. Use `#[require_group(...)]` for single group checks
2. Use `#[require_any_groups(...)]` for OR logic
3. Use `#[require_all_groups(...)]` for AND logic
4. Stack macros for complex authorization
5. No more manual guard code!

## See Also

- [Phase 2B Complete](PHASE2B_IMPLEMENTATION_COMPLETE.md) - Technical details
- [Phase 2 Quick Reference](PHASE2_QUICK_REFERENCE.md) - Guards and extraction
- [Current Status](CURRENT_STATUS.md) - Overall project status

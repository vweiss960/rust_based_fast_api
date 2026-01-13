# Phase 2 Implementation Plan - Authorization Macros & Extractors

## Overview

Phase 2 focuses on making authorization and user claims extraction automatic and declarative, eliminating the need for manual token extraction and permission checking in every handler.

## Goals

1. **Authorization Guard Macros** - Declarative permission checking
2. **Poem FromRequest Extractor** - Automatic UserClaims extraction
3. **Reduce Handler Boilerplate** - Simplify authentication code

## Current State (After Phase 1)

### Current Code Pattern
```rust
#[handler]
async fn protected(req: &poem::Request) -> Response {
    let state = PoemAppState::get();

    match extract_jwt_from_request(req) {
        Ok(token) => {
            match state.jwt.verify_token(&token) {
                Ok(claims) => {
                    // Finally have claims...
                    if claims.has_group("admin") {
                        // Do admin stuff
                    }
                }
                Err(_) => /* error response */
            }
        }
        Err(_) => /* error response */
    }
}
```

**Issues:**
- 15+ lines of boilerplate per handler
- Manual token extraction every time
- Manual permission checking
- Repeated error handling

## Phase 2 Solution

### Goal: One-line Handlers

```rust
#[require_groups("admin")]
#[handler]
async fn delete_user(claims: UserClaims, Json(req): Json<DeleteRequest>) -> Response {
    // claims are automatically extracted and validated
    // permission already checked (or 403 returned automatically)
    delete_user_from_db(&req.username).await
}
```

## Implementation Strategy

### Part 1: Poem FromRequest Extractor

**File**: `src/poem_integration/extractors.rs` (NEW)

```rust
use poem::{FromRequest, RequestParts};
use crate::auth::UserClaims;
use crate::poem_integration::PoemAppState;

/// Automatic extractor for UserClaims from JWT in Authorization header
#[async_trait::async_trait]
impl<'a> FromRequest<'a> for UserClaims {
    async fn from_request(req: &'a mut RequestParts) -> Result<Self, poem::Error> {
        let state = PoemAppState::get();

        let auth_header = req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                poem::Error::from_status(StatusCode::UNAUTHORIZED)
            })?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                poem::Error::from_status(StatusCode::UNAUTHORIZED)
            })?;

        state.jwt.verify_token(token)
            .map_err(|_| poem::Error::from_status(StatusCode::UNAUTHORIZED))
    }
}
```

**Usage**:
```rust
#[handler]
async fn protected(claims: UserClaims) -> String {
    // claims automatically extracted and validated!
    format!("Hello {}", claims.sub)
}
```

### Part 2: Authorization Guard Macros

**New Crate**: `poem_auth_macros` (procedural macro crate)

#### Macro 1: `#[require_groups("role1", "role2")]` (AND logic)

```rust
/// Requires user to have ALL specified groups
#[require_groups("admin", "verified")]
#[handler]
async fn restricted_endpoint(claims: UserClaims) -> String {
    format!("User {} has admin + verified", claims.sub)
}
```

**Expands to**:
```rust
#[handler]
async fn restricted_endpoint(claims: UserClaims) -> Result<String, StatusCode> {
    if !claims.has_all_groups(&["admin", "verified"]) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(format!("User {} has admin + verified", claims.sub))
}
```

#### Macro 2: `#[require_any_groups("role1", "role2")]` (OR logic)

```rust
/// Requires user to have ANY of the specified groups
#[require_any_groups("admin", "moderator")]
#[handler]
async fn moderate(claims: UserClaims) -> String {
    format!("Moderator {}", claims.sub)
}
```

**Expands to**:
```rust
#[handler]
async fn moderate(claims: UserClaims) -> Result<String, StatusCode> {
    if !claims.has_any_group(&["admin", "moderator"]) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(format!("Moderator {}", claims.sub))
}
```

#### Macro 3: `#[require_group("role")]` (Single group)

```rust
/// Requires user to have a specific group
#[require_group("admin")]
#[handler]
async fn admin_only(claims: UserClaims) -> String {
    "Admin access".to_string()
}
```

### Part 3: Guard Trait Pattern (Alternative to Macros)

For users who prefer guards over macros:

**File**: `src/poem_integration/guards.rs` (NEW)

```rust
pub trait AuthGuard: Send + Sync {
    fn check(&self, claims: &UserClaims) -> bool;
}

pub struct HasGroup(pub String);

impl AuthGuard for HasGroup {
    fn check(&self, claims: &UserClaims) -> bool {
        claims.has_group(&self.0)
    }
}

pub struct HasAnyGroup(pub Vec<String>);

impl AuthGuard for HasAnyGroup {
    fn check(&self, claims: &UserClaims) -> bool {
        claims.has_any_group(&self.0)
    }
}

pub struct HasAllGroups(pub Vec<String>);

impl AuthGuard for HasAllGroups {
    fn check(&self, claims: &UserClaims) -> bool {
        claims.has_all_groups(&self.0)
    }
}

// Composable guards
pub struct And<A, B>(pub A, pub B);

impl<A: AuthGuard, B: AuthGuard> AuthGuard for And<A, B> {
    fn check(&self, claims: &UserClaims) -> bool {
        self.0.check(claims) && self.1.check(claims)
    }
}

pub struct Or<A, B>(pub A, pub B);

impl<A: AuthGuard, B: AuthGuard> AuthGuard for Or<A, B> {
    fn check(&self, claims: &UserClaims) -> bool {
        self.0.check(claims) || self.1.check(claims)
    }
}
```

**Usage**:
```rust
#[handler]
async fn protected(claims: UserClaims, req: &poem::Request) -> Result<String, StatusCode> {
    let guard = And(HasGroup("admin".to_string()), HasGroup("verified".to_string()));

    if !guard.check(&claims) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok("Access granted".to_string())
}
```

## Implementation Plan

### Step 1: Add FromRequest Extractor
- File: `src/poem_integration/extractors.rs`
- Implement `FromRequest` for `UserClaims`
- Export from `poem_integration::mod.rs`
- Add to prelude

### Step 2: Create Macro Crate
- New crate: `poem_auth_macros/`
- Setup procedural macro infrastructure
- Implement macros using `quote!` and `syn`
- Add tests

### Step 3: Implement Guard Trait
- File: `src/poem_integration/guards.rs`
- AuthGuard trait + implementations
- Composable guards with And/Or
- Export from `poem_integration::mod.rs`

### Step 4: Update Examples
- Update `examples/poem_example/` to use extractors
- Add example handlers with macros
- Demonstrate all authorization patterns

### Step 5: Documentation
- Add examples to extractors
- Add examples to guard implementations
- Create usage guide for macros

## Expected Outcomes

### Before Phase 2
```rust
#[handler]
async fn protected(req: &poem::Request) -> Response {
    let state = PoemAppState::get();
    let token = extract_token(req)?;
    let claims = state.jwt.verify_token(&token)?;

    if !claims.has_group("admin") {
        return Err(StatusCode::FORBIDDEN);
    }

    // Finally do work...
}
```

### After Phase 2
```rust
#[require_group("admin")]
#[handler]
async fn protected(claims: UserClaims) -> String {
    "Access granted".to_string()
}
```

**Result**: 13 lines → 3 lines (77% reduction)

## Backwards Compatibility

All Phase 2 features are **additive**:
- Old code continues to work
- Users can opt-in to macros/extractors
- Mix old and new patterns in same app

## Testing Strategy

1. Unit tests for guards
2. Integration tests with poem_example
3. Macro expansion tests
4. Error case testing (missing auth, invalid token, insufficient permissions)

## Success Criteria

✅ UserClaims automatically extracted from JWT in handlers
✅ Authorization macros work and properly restrict access
✅ 70%+ boilerplate reduction in typical handlers
✅ All examples compile and run
✅ Documentation complete with usage examples
✅ Backwards compatible with Phase 1 code

## Timeline Estimate

- Part 1 (Extractors): 1-2 hours
- Part 2 (Macros): 2-3 hours
- Part 3 (Guards): 1-2 hours
- Testing & Examples: 1-2 hours
- **Total**: 5-9 hours

## Files to Create/Modify

**New Files**:
- `src/poem_integration/extractors.rs` - FromRequest impl
- `src/poem_integration/guards.rs` - Guard trait + impls
- `poem_auth_macros/Cargo.toml` - Macro crate config
- `poem_auth_macros/src/lib.rs` - Macro implementations
- `poem_auth_macros/src/guard_macros.rs` - Guard macro logic

**Modified Files**:
- `src/lib.rs` - Export extractors and guards
- `src/poem_integration/mod.rs` - Export extractors and guards
- `Cargo.toml` - Add macro crate dependency
- `examples/poem_example/src/main.rs` - Use new features

## Notes

- **Macro crate**: Procedural macros must be in separate crate
- **Async trait**: Need `async_trait` for async FromRequest
- **Error types**: Must return poem::Error from extractors
- **Composition**: Guards designed to be composable
- **Flexibility**: Both macros and guards available for different preferences

---

## Next Phase Preview (Phase 3)

After Phase 2 completes, Phase 3 will add:
- Admin endpoint generator (pre-built CRUD routes)
- Typed claims builder (type-safe custom claims)

This will eliminate need for users to write any admin endpoints.

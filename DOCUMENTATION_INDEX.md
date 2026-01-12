# Documentation Index

Welcome to poem_auth! This index will help you find what you need.

## Quick Navigation

### I'm brand new to poem_auth
👉 Start here: **[GETTING_STARTED.md](GETTING_STARTED.md)**
- Step-by-step walkthrough
- Your first authentication app
- Basic testing examples

### I need to see all available APIs
👉 Read: **[API_REFERENCE.md](API_REFERENCE.md)**
- Complete type reference
- All methods and their signatures
- Example usage for each API

### I want detailed usage examples
👉 Check: **[README.md](README.md)**
- Feature overview
- Usage patterns for different scenarios
- Local auth, LDAP, custom providers
- JWT management
- Middleware integration
- User management APIs
- Security best practices

### I need help solving a problem
👉 See: **[PATTERNS_AND_TROUBLESHOOTING.md](PATTERNS_AND_TROUBLESHOOTING.md)**
- Common implementation patterns
- Solutions to frequent issues
- Testing strategies
- Performance optimization
- Security checklist

---

## Learning Path

### Beginner (30 minutes)
1. Read [GETTING_STARTED.md](GETTING_STARTED.md)
2. Run the basic example
3. Test with curl

### Intermediate (1-2 hours)
1. Review [README.md](README.md) "Usage Guides" section
2. Try local authentication example
3. Implement a login endpoint
4. Add protected routes

### Advanced (2-4 hours)
1. Study [API_REFERENCE.md](API_REFERENCE.md)
2. Implement custom `AuthProvider`
3. Add role-based access control
4. Set up LDAP integration
5. Review [PATTERNS_AND_TROUBLESHOOTING.md](PATTERNS_AND_TROUBLESHOOTING.md)

### Expert (Ongoing)
1. Customize database backend
2. Implement token refresh flow
3. Design audit logging
4. Optimize for production use

---

## Documentation by Task

### Setting Up Authentication

**For local database:**
- [GETTING_STARTED.md](GETTING_STARTED.md) - Quick setup
- [README.md#local-authentication](README.md#local-authentication) - Detailed guide
- [API_REFERENCE.md#localautprovider](API_REFERENCE.md#localautprovider) - API docs

**For LDAP/Active Directory:**
- [README.md#ldapactive-directory](README.md#ldapactive-directory) - Setup guide
- [API_REFERENCE.md#ldapauthprovider](API_REFERENCE.md#ldapauthprovider) - Configuration options

### Building Protected Routes

- [README.md#middleware-integration](README.md#middleware-integration) - Overview
- [API_REFERENCE.md#middleware](API_REFERENCE.md#middleware) - All middleware types
- [PATTERNS_AND_TROUBLESHOOTING.md](PATTERNS_AND_TROUBLESHOOTING.md) - Role-based access patterns

### Managing Users

- [README.md#user-management-apis](README.md#user-management-apis) - API examples
- [API_REFERENCE.md#userdatabase-trait](API_REFERENCE.md#userdatabase-trait) - Database interface
- [PATTERNS_AND_TROUBLESHOOTING.md#pattern-6-user-management-endpoints](PATTERNS_AND_TROUBLESHOOTING.md#pattern-6-user-management-endpoints) - Full implementation

### Token Management

- [README.md#jwt--token-management](README.md#jwt--token-management) - Token overview
- [API_REFERENCE.md#jwtvalidator](API_REFERENCE.md#jwtvalidator) - API reference
- [PATTERNS_AND_TROUBLESHOOTING.md#pattern-4-token-refresh](PATTERNS_AND_TROUBLESHOOTING.md#pattern-4-token-refresh) - Refresh tokens

### Custom Authentication

- [README.md#custom-authentication-providers](README.md#custom-authentication-providers) - Guide
- [API_REFERENCE.md#authprovider-trait](API_REFERENCE.md#authprovider-trait) - Trait definition
- [PATTERNS_AND_TROUBLESHOOTING.md](PATTERNS_AND_TROUBLESHOOTING.md) - Common implementations

### Troubleshooting

- [PATTERNS_AND_TROUBLESHOOTING.md#troubleshooting](PATTERNS_AND_TROUBLESHOOTING.md#troubleshooting) - Problem solving guide
- [README.md#security](README.md#security) - Security best practices
- [PATTERNS_AND_TROUBLESHOOTING.md#testing-patterns](PATTERNS_AND_TROUBLESHOOTING.md#testing-patterns) - How to test

---

## API Quick Reference

### Core Types

| Type | Purpose | Location |
|------|---------|----------|
| `UserClaims` | Authenticated user info in tokens | [API_REFERENCE.md#userclaims](API_REFERENCE.md#userclaims) |
| `UserRecord` | User in database | [API_REFERENCE.md#userrecord](API_REFERENCE.md#userrecord) |
| `AuthProvider` | Auth method interface | [API_REFERENCE.md#authprovider-trait](API_REFERENCE.md#authprovider-trait) |
| `UserDatabase` | User storage interface | [API_REFERENCE.md#userdatabase-trait](API_REFERENCE.md#userdatabase-trait) |

### Implementations

| Implementation | Purpose | Feature | Location |
|---|---|---|---|
| `LocalAuthProvider` | Local DB authentication | default | [API_REFERENCE.md#localauthprovider](API_REFERENCE.md#localauthprovider) |
| `LdapAuthProvider` | LDAP/AD authentication | `ldap` | [API_REFERENCE.md#ldapauthprovider](API_REFERENCE.md#ldapauthprovider) |
| `SqliteUserDb` | SQLite database | `sqlite` | [API_REFERENCE.md#sqliteuserdb](API_REFERENCE.md#sqliteuserdb) |
| `JwtValidator` | JWT operations | default | [API_REFERENCE.md#jwtvalidator](API_REFERENCE.md#jwtvalidator) |
| `TokenCache` | Token caching | `cache` | [API_REFERENCE.md#tokencache](API_REFERENCE.md#tokencache) |
| `RateLimit` | Rate limiting | `rate-limit` | [API_REFERENCE.md#ratelimit-and-ratelimitconfig](API_REFERENCE.md#ratelimit-and-ratelimitconfig) |

### Key Functions

| Function | Purpose | Location |
|----------|---------|----------|
| `hash_password()` | Hash a password | [API_REFERENCE.md#password-hashing](API_REFERENCE.md#password-hashing) |
| `verify_password()` | Verify password hash | [API_REFERENCE.md#password-hashing](API_REFERENCE.md#password-hashing) |
| `extract_jwt_claims()` | Extract claims from request | [API_REFERENCE.md#extract_jwt_claims](API_REFERENCE.md#extract_jwt_claims) |

---

## Common Code Snippets

### Create a User

```rust
use poem_auth::db::UserRecord;
use poem_auth::password::hash_password;

let hash = hash_password("password123")?;
let user = UserRecord::new("alice", &hash)
    .with_groups(vec!["users"]);
db.create_user(user).await?;
```

### Authenticate User

```rust
let claims = provider.authenticate("alice", "password123").await?;
```

### Check Permissions

```rust
if claims.has_group("admins") {
    // Allow admin action
}
```

### Create Token

```rust
let token = jwt.encode(&claims)?;
```

### Verify Token

```rust
let claims = jwt.decode(&token)?;
```

---

## Code Organization

```
src/
├── auth/                 # Core authentication
│   ├── provider.rs       # AuthProvider trait
│   └── claims.rs         # UserClaims struct
├── db/                   # Database abstractions
│   ├── models.rs         # UserDatabase trait
│   └── sqlite.rs         # SQLite implementation
├── providers/            # Auth method implementations
│   ├── local.rs          # Local auth
│   ├── ldap.rs           # LDAP/AD auth
│   └── mod.rs
├── jwt/                  # JWT handling
│   ├── jwt.rs            # JwtValidator
│   ├── cache.rs          # Token caching
│   └── mod.rs
├── middleware/           # Poem middleware
│   ├── jwt_auth.rs       # JWT extraction
│   ├── master_auth.rs    # Master auth
│   ├── rate_limit.rs     # Rate limiting
│   └── mod.rs
├── api/                  # HTTP API types
│   ├── types.rs          # Request/response DTOs
│   └── mod.rs
├── password.rs           # Password utilities
├── error.rs              # Error types
└── lib.rs                # Library entry point
```

---

## Feature Flags

Enable features in `Cargo.toml`:

```toml
# Minimal (just core auth)
poem_auth = "0.1"

# With SQLite (default)
poem_auth = { version = "0.1", features = ["sqlite"] }

# Full featured
poem_auth = { version = "0.1", features = ["sqlite", "ldap", "cache", "rate-limit", "cors"] }
```

| Feature | Default | Required For |
|---------|---------|---|
| `sqlite` | Yes | Local user database |
| `cache` | Yes | Token caching |
| `ldap` | No | LDAP/AD authentication |
| `rate-limit` | No | Rate limiting middleware |
| `cors` | No | CORS support |
| `cli` | No | CLI utilities |

---

## When to Read What

### "How do I...?"

| Question | Document |
|----------|----------|
| Get started? | [GETTING_STARTED.md](GETTING_STARTED.md) |
| Use local authentication? | [README.md#local-authentication](README.md#local-authentication) |
| Set up LDAP? | [README.md#ldapactive-directory](README.md#ldapactive-directory) |
| Protect a route? | [README.md#middleware-integration](README.md#middleware-integration) |
| Manage users? | [README.md#user-management-apis](README.md#user-management-apis) |
| Implement a custom auth method? | [README.md#custom-authentication-providers](README.md#custom-authentication-providers) |
| Check user permissions? | [PATTERNS_AND_TROUBLESHOOTING.md#pattern-2-role-based-access-control-rbac](PATTERNS_AND_TROUBLESHOOTING.md#pattern-2-role-based-access-control-rbac) |
| Fix an error? | [PATTERNS_AND_TROUBLESHOOTING.md#troubleshooting](PATTERNS_AND_TROUBLESHOOTING.md#troubleshooting) |
| Find an API signature? | [API_REFERENCE.md](API_REFERENCE.md) |
| Implement a pattern? | [PATTERNS_AND_TROUBLESHOOTING.md#common-patterns](PATTERNS_AND_TROUBLESHOOTING.md#common-patterns) |

---

## Getting Help

### Inside This Documentation

1. **Check the index above** - Does your question match any entry?
2. **Search for keywords** - Most documents are searchable
3. **Read examples** - Look at code examples in the document
4. **Check error messages** - Error types often suggest solutions

### In Your Code

```rust
// Read the docs for any type:
// cargo doc --open

// Check error types:
use poem_auth::error::AuthError;

// Use the prelude for common types:
use poem_auth::prelude::*;
```

### Common First Steps

1. **"It doesn't compile"** → Check [API_REFERENCE.md](API_REFERENCE.md) for correct signatures
2. **"It doesn't work"** → See [PATTERNS_AND_TROUBLESHOOTING.md#troubleshooting](PATTERNS_AND_TROUBLESHOOTING.md#troubleshooting)
3. **"What's the API?"** → Read [API_REFERENCE.md](API_REFERENCE.md)
4. **"How do I do X?"** → Check "When to Read What" table above

---

## Document Sizes & Reading Time

| Document | Size | Time |
|----------|------|------|
| [GETTING_STARTED.md](GETTING_STARTED.md) | ~15 KB | 15-20 min |
| [README.md](README.md) | ~50 KB | 40-60 min |
| [API_REFERENCE.md](API_REFERENCE.md) | ~40 KB | Reference |
| [PATTERNS_AND_TROUBLESHOOTING.md](PATTERNS_AND_TROUBLESHOOTING.md) | ~35 KB | Reference |

---

## Changelog & Updates

This documentation covers **poem_auth v0.1.0**

See [CHANGELOG.md](CHANGELOG.md) for version history and breaking changes.

---

## Contributing Documentation

Found an error? Missing example? Unclear section?

- Open an issue on GitHub
- Submit a pull request with improvements
- Share feedback on what's confusing

---

**Happy authenticating! 🔐**

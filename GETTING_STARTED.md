# Getting Started with poem_auth

This guide walks you through your first authentication-enabled Poem application.

## Prerequisites

- Rust 1.70+ (via [rustup](https://rustup.rs/))
- Basic Rust knowledge
- Familiarity with async/await

## Step 1: Create a New Project

```bash
cargo new my_auth_app
cd my_auth_app
```

## Step 2: Add Dependencies

Edit your `Cargo.toml`:

```toml
[package]
name = "my_auth_app"
version = "0.1.0"
edition = "2021"

[dependencies]
poem = { version = "3", features = ["tower"] }
poem_auth = { version = "0.1", features = ["sqlite"] }
tokio = { version = "1", features = ["full"] }
serde_json = "1"
chrono = "0.4"
```

## Step 3: Create Your First App

Create `src/main.rs`:

```rust
use poem::{listener::TcpListener, App, get, Route};
use poem_auth::prelude::*;
use poem_auth::db::sqlite::SqliteUserDb;
use poem_auth::providers::LocalAuthProvider;
use poem_auth::jwt::JwtValidator;
use poem_auth::db::UserRecord;
use poem_auth::password::hash_password;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing authentication system...");

    // Step 1: Set up the database
    let db = SqliteUserDb::new("users.db").await?;
    println!("✓ Database initialized");

    // Step 2: Create test user
    // Only create if it doesn't exist
    match db.get_user("alice").await {
        Err(_) => {
            let password_hash = hash_password("password123")?;
            let user = UserRecord::new("alice", &password_hash)
                .with_groups(vec!["users", "developers"]);
            db.create_user(user).await?;
            println!("✓ Test user 'alice' created (password: password123)");
        }
        Ok(_) => {
            println!("✓ Test user 'alice' already exists");
        }
    }

    // Step 3: Create authentication provider
    let provider = LocalAuthProvider::new(db.clone());

    // Step 4: Create JWT validator
    let jwt = JwtValidator::new("my-super-secret-key-should-be-at-least-16-chars")?;

    // Step 5: Build the Poem app
    let app = App::new()
        .at("/health", get(health_check))
        .at("/public", get(public_handler))
        .at("/protected", get(protected_handler));

    // Step 6: Run the server
    let addr = "127.0.0.1:3000";
    println!("🚀 Server running at http://{}", addr);
    println!("\nTry these endpoints:");
    println!("  GET http://{}/health", addr);
    println!("  GET http://{}/public", addr);
    println!("  GET http://{}/protected (requires bearer token)", addr);
    println!("\n📝 To get a token:");
    println!("  curl -X POST http://{}/login \\", addr);
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"username\":\"alice\",\"password\":\"password123\"}}'");

    app.run(TcpListener::bind(addr)).await?;
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn public_handler() -> String {
    "This endpoint is public - no authentication required".to_string()
}

async fn protected_handler(claims: UserClaims) -> String {
    format!(
        "Hello {}! You are authenticated.\nYour groups: {:?}",
        claims.sub, claims.groups
    )
}
```

## Step 4: Run Your App

```bash
cargo run
```

You should see:
```
Initializing authentication system...
✓ Database initialized
✓ Test user 'alice' already exists
🚀 Server running at http://127.0.0.1:3000
```

## Step 5: Test the Endpoints

### Test the health endpoint:
```bash
curl http://127.0.0.1:3000/health
# Output: OK
```

### Test the public endpoint:
```bash
curl http://127.0.0.1:3000/public
# Output: This endpoint is public - no authentication required
```

### Test the protected endpoint (without token - should fail):
```bash
curl http://127.0.0.1:3000/protected
# Output: 401 Unauthorized
```

## Step 6: Get a Token

First, we need to add a login endpoint. Update `src/main.rs`:

```rust
use poem::{listener::TcpListener, App, get, post, Route, web::Json};
use poem_auth::api::types::{LoginRequest, LoginResponse, ErrorResponse, UserClaimsResponse};
use poem_auth::prelude::*;
use poem_auth::db::sqlite::SqliteUserDb;
use poem_auth::providers::LocalAuthProvider;
use poem_auth::jwt::JwtValidator;
use poem_auth::db::UserRecord;
use poem_auth::password::hash_password;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing authentication system...");

    let db = SqliteUserDb::new("users.db").await?;
    println!("✓ Database initialized");

    match db.get_user("alice").await {
        Err(_) => {
            let password_hash = hash_password("password123")?;
            let user = UserRecord::new("alice", &password_hash)
                .with_groups(vec!["users", "developers"]);
            db.create_user(user).await?;
            println!("✓ Test user 'alice' created (password: password123)");
        }
        Ok(_) => {
            println!("✓ Test user 'alice' already exists");
        }
    }

    let provider = LocalAuthProvider::new(db.clone());
    let jwt = JwtValidator::new("my-super-secret-key-should-be-at-least-16-chars")?;

    // Wrap in Arc to share across handlers
    let provider = Arc::new(provider);
    let jwt = Arc::new(jwt);

    let app = App::new()
        .at("/health", get(health_check))
        .at("/public", get(public_handler))
        .at("/login", post({
            let provider = provider.clone();
            let jwt = jwt.clone();
            move |req| login(req, provider.clone(), jwt.clone())
        }))
        .at("/protected", get(protected_handler));

    let addr = "127.0.0.1:3000";
    println!("🚀 Server running at http://{}", addr);
    println!("\n📝 Login example:");
    println!("  curl -X POST http://{}/login \\", addr);
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"username\":\"alice\",\"password\":\"password123\"}}'");

    app.run(TcpListener::bind(addr)).await?;
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn public_handler() -> String {
    "This endpoint is public".to_string()
}

async fn login(
    req: Json<LoginRequest>,
    provider: Arc<LocalAuthProvider>,
    jwt: Arc<JwtValidator>,
) -> Result<Json<LoginResponse>, Json<ErrorResponse>> {
    // Authenticate user
    let claims = provider.authenticate(&req.username, &req.password)
        .await
        .map_err(|e| {
            eprintln!("Auth error: {:?}", e);
            Json(ErrorResponse::invalid_credentials())
        })?;

    // Generate token
    let token = jwt.encode(&claims)
        .map_err(|e| {
            eprintln!("Token error: {:?}", e);
            Json(ErrorResponse::new("error", "Failed to generate token"))
        })?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: claims.exp - claims.iat,
        claims: UserClaimsResponse::from_claims(claims),
    }))
}

async fn protected_handler(claims: UserClaims) -> String {
    format!(
        "Hello {}! You have access.\nYour groups: {:?}",
        claims.sub, claims.groups
    )
}
```

Now get a token:

```bash
curl -X POST http://127.0.0.1:3000/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"password123"}'
```

Response:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "claims": {
    "sub": "alice",
    "provider": "local",
    "groups": ["users", "developers"],
    "exp": 1704153600,
    "iat": 1704067200
  }
}
```

## Step 7: Access Protected Endpoint

Use the token from the login response:

```bash
TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

curl http://127.0.0.1:3000/protected \
  -H "Authorization: Bearer $TOKEN"
```

Response:
```
Hello alice! You have access.
Your groups: ["users", "developers"]
```

## Next Steps

### Add Role-Based Access Control (RBAC)

```rust
async fn admin_handler(claims: UserClaims) -> Result<String, String> {
    if !claims.has_group("admins") {
        return Err("Admin access required".to_string());
    }
    Ok("Admin panel".to_string())
}
```

### Create More Users

```rust
async fn create_user(
    req: Json<CreateUserRequest>,
    db: &SqliteUserDb,
) -> Result<String, String> {
    let hash = hash_password(&req.password)
        .map_err(|e| e.to_string())?;

    let user = UserRecord::new(&req.username, &hash)
        .with_groups(req.groups);

    db.create_user(user).await
        .map_err(|e| e.to_string())?;

    Ok(format!("User {} created", req.username))
}
```

### Enable LDAP Authentication

Add the `ldap` feature:

```toml
poem_auth = { version = "0.1", features = ["sqlite", "ldap"] }
```

Then use `LdapAuthProvider`:

```rust
use poem_auth::providers::{LdapAuthProvider, LdapConfig};

let ldap_config = LdapConfig {
    server_url: "ldap://your-ldap-server:389".to_string(),
    bind_dn_template: "uid={username},ou=people,dc=example,dc=com".to_string(),
    // ... other config
};

let ldap_provider = LdapAuthProvider::new(ldap_config)?;
```

### Enable Rate Limiting

Add the `rate-limit` feature:

```toml
poem_auth = { version = "0.1", features = ["sqlite", "rate-limit"] }
```

### Common Issues

**Issue**: "Database is locked"
- Ensure only one instance is accessing the database
- Use a database connection pool in production

**Issue**: Token validation fails
- Check that you're using the same JWT secret for encoding and decoding
- Ensure token hasn't expired

**Issue**: User not found after creating
- Verify the database file exists and is writable
- Check your username exactly matches

## Resources

- [README.md](README.md) - Complete feature documentation
- [Example Files](examples/) - More detailed examples
- API Docs: `cargo doc --open`

## Getting Help

- Check the [README.md](README.md) for detailed API documentation
- Review source code comments: `cargo doc --open`
- Check error messages carefully - they're designed to be helpful

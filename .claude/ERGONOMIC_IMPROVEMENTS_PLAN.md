# poem_auth Ergonomic Improvements Plan

## Goal
Reduce boilerplate and complexity when setting up poem_auth with Poem, allowing users to go from 50+ lines of setup code to <15 lines.

## Summary of Recommended Changes

| Feature | Lines Saved | Pain Points Fixed |
|---------|------------|------------------|
| AppState export | 10-15 | Global state management |
| Config file support | 30-50 | User creation, path management |
| Authorization macros | 30-50 | Repetitive group checks |
| Middleware/Extractor | 15-20 | Manual token extraction |
| Admin routes | 100+ | Standard CRUD endpoints |
| Quick-start init | 35-50 | Setup complexity |
| **TOTAL** | **~220-295 lines** | **Reduces setup from 50 lines to <15 lines** |

---

## Implementation Phases

### Phase 1 (Immediate - High Impact) ✅ START HERE
1. **PoemAppState builder** - Exportable, reusable state struct
2. **Configuration file support (TOML)** - Load users, database, JWT settings from config
3. **Quick-start initializer** - One-liner setup function

**Expected Outcome:** poem_example reduces from ~200 lines to ~50 lines

### Phase 2 (Short-term - Ergonomics)
4. **Authorization guard macros** (separate crate: `poem_auth_macros`)
5. **Poem Middleware/Extractor wrappers** - Automatic UserClaims extraction

**Expected Outcome:** Handler functions become much simpler, no manual token extraction

### Phase 3 (Medium-term - Features)
6. **Admin endpoint generator** - Pre-built CRUD routes
7. **Typed claims builder** - Type-safe custom claims

**Expected Outcome:** Admin endpoints require no custom code

### Phase 4 (Later - Polish)
8. **Audit logging abstraction** - Pluggable audit backends
9. **Token refresh management** - Automatic token refresh endpoints
10. **Rate limiting integration** - Per-endpoint rate limit configuration

---

## Phase 1 Detailed Design

### 1. PoemAppState Builder

**File:** `src/poem_integration/app_state.rs` (NEW)

```rust
use std::sync::OnceLock;
use crate::providers::LocalAuthProvider;
use crate::jwt::JwtValidator;
use std::sync::Arc;

/// Shared application state for Poem web server
#[derive(Clone)]
pub struct PoemAppState {
    pub provider: Arc<LocalAuthProvider>,
    pub jwt: Arc<JwtValidator>,
}

static APP_STATE: OnceLock<PoemAppState> = OnceLock::new();

impl PoemAppState {
    /// Create a new PoemAppState with database and JWT secret
    pub async fn new(
        db_path: &str,
        jwt_secret: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let db = crate::db::sqlite::SqliteUserDb::new(db_path).await?;
        let provider = Arc::new(LocalAuthProvider::new(db));
        let jwt = Arc::new(JwtValidator::new(jwt_secret)?);

        Ok(PoemAppState { provider, jwt })
    }

    /// Initialize global app state (call once during app startup)
    pub fn init(self) -> Result<(), Self> {
        APP_STATE.set(self)
    }

    /// Get reference to global app state (panics if not initialized)
    pub fn get() -> &'static PoemAppState {
        APP_STATE.get().expect("PoemAppState not initialized. Call PoemAppState::init() first.")
    }

    /// Try to get reference to global app state
    pub fn try_get() -> Option<&'static PoemAppState> {
        APP_STATE.get()
    }

    /// Get shared reference for cloning into Poem state
    pub fn provider(&self) -> Arc<LocalAuthProvider> {
        self.provider.clone()
    }

    pub fn jwt(&self) -> Arc<JwtValidator> {
        self.jwt.clone()
    }
}
```

**Usage:**
```rust
// Old: 5+ lines with OnceLock
static APP_STATE: OnceLock<AppState> = OnceLock::new();

// New: 1 line per state creation
let app_state = PoemAppState::new("users.db", "my-secret").await?;
app_state.init()?;

// In handlers:
let state = PoemAppState::get();
```

---

### 2. Configuration File Support (TOML)

**File:** `src/config.rs` (NEW)

```rust
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub users: Vec<UserConfig>,
    #[serde(default)]
    pub server: Option<ServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    #[serde(default = "default_auto_create")]
    pub auto_create: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    #[serde(default = "default_expiration_hours")]
    pub expiration_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

fn default_auto_create() -> bool { true }
fn default_expiration_hours() -> u32 { 24 }
fn default_enabled() -> bool { true }

impl AuthConfig {
    /// Load configuration from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load from environment variables with fallback to file
    pub fn from_env_or_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(config_str) = std::env::var("AUTH_CONFIG") {
            let config = toml::from_str(&config_str)?;
            Ok(config)
        } else {
            Self::from_file(file_path)
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.jwt.secret.len() < 16 {
            return Err("JWT secret must be at least 16 characters".to_string());
        }
        if self.database.path.is_empty() {
            return Err("Database path cannot be empty".to_string());
        }
        Ok(())
    }
}
```

**Dependencies to add to Cargo.toml:**
```toml
toml = "0.8"
```

**Example auth.toml:**
```toml
[database]
path = "users.db"
auto_create = true

[jwt]
secret = "my-super-secret-key-should-be-at-least-16-chars"
expiration_hours = 24

[[users]]
username = "alice"
password = "password123"
groups = ["users"]
enabled = true

[[users]]
username = "bob"
password = "secret456"
groups = ["users", "admins"]
enabled = true

[server]
host = "0.0.0.0"
port = 3000
```

---

### 3. Quick-Start Initializer

**File:** `src/quick_start.rs` (NEW)

```rust
use crate::config::AuthConfig;
use crate::db::sqlite::SqliteUserDb;
use crate::db::UserRecord;
use crate::password::hash_password;
use crate::providers::LocalAuthProvider;
use crate::jwt::JwtValidator;
use crate::poem_integration::PoemAppState;
use std::sync::Arc;

/// Initialize authentication system from config file
///
/// This function:
/// 1. Loads configuration from TOML file
/// 2. Creates/validates SQLite database
/// 3. Creates users from config if they don't exist
/// 4. Initializes auth components (LocalAuthProvider, JwtValidator)
/// 5. Sets up global PoemAppState
///
/// # Example
/// ```ignore
/// let app_state = initialize_from_config("auth.toml").await?;
///
/// let app = Route::new()
///     .at("/login", post(login))
///     .at("/protected", get(protected));
/// ```
pub async fn initialize_from_config(
    config_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config = AuthConfig::from_file(config_path)?;
    config.validate()?;

    println!("=== Initializing Authentication System ===\n");

    // Initialize database
    println!("Step 1: Initialize database at '{}'", config.database.path);
    let db = SqliteUserDb::new(&config.database.path).await?;
    println!("✓ Database initialized\n");

    // Create users from config
    println!("Step 2: Create users from configuration");
    for user_config in &config.users {
        match db.get_user(&user_config.username).await {
            Ok(_) => {
                println!("  - {} (already exists)", user_config.username);
            }
            Err(_) => {
                let hash = hash_password(&user_config.password)?;
                let mut user = UserRecord::new(&user_config.username, &hash);

                if !user_config.groups.is_empty() {
                    user = user.with_groups(user_config.groups.clone());
                }

                if !user_config.enabled {
                    user = user.disable();
                }

                db.create_user(user).await?;
                println!("  ✓ Created: {} with groups {:?}",
                    user_config.username, user_config.groups);
            }
        }
    }
    println!();

    // Create auth components
    println!("Step 3: Create authentication components");
    let provider = Arc::new(LocalAuthProvider::new(db));
    let jwt = Arc::new(JwtValidator::new(&config.jwt.secret)?);
    println!("✓ LocalAuthProvider created");
    println!("✓ JwtValidator created\n");

    // Initialize global state
    let app_state = PoemAppState {
        provider,
        jwt,
    };
    app_state.init()?;

    // Print summary
    println!("✅ Authentication system initialized successfully!");
    println!("\nServer configuration:");
    if let Some(server) = &config.server {
        println!("  - Address: {}:{}", server.host, server.port);
    }
    println!("  - JWT Secret: {}...{}",
        &config.jwt.secret[..8],
        &config.jwt.secret[config.jwt.secret.len()-4..]);
    println!("  - Expiration: {} hours", config.jwt.expiration_hours);
    println!("  - Users: {}", config.users.len());
    println!();

    Ok(())
}
```

---

## Module Structure

Add to `src/lib.rs`:
```rust
pub mod config;
pub mod quick_start;
pub mod poem_integration;

pub use config::AuthConfig;
pub use quick_start::initialize_from_config;
pub use poem_integration::PoemAppState;
```

---

## Before and After Comparison

### BEFORE (Current poem_example)
```rust
// ~200 lines of setup code
static APP_STATE: OnceLock<AppState> = OnceLock::new();

#[derive(Clone)]
pub struct AppState {
    pub provider: Arc<LocalAuthProvider>,
    pub jwt: Arc<JwtValidator>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = SqliteUserDb::new("poem_example.db").await?;

    for (username, password) in &[("alice", "password123"), ("bob", "secret456")] {
        if db.get_user(username).await.is_err() {
            let hash = hash_password(password)?;
            let user = UserRecord::new(username, &hash)
                .with_groups(vec!["users"]);
            db.create_user(user).await?;
        }
    }

    let provider = Arc::new(LocalAuthProvider::new(db));
    let jwt = Arc::new(JwtValidator::new("secret")?);

    let app_state = AppState {
        provider: provider.clone(),
        jwt: jwt.clone(),
    };
    APP_STATE.get_or_init(|| app_state);

    // ... then route definitions
}
```

### AFTER (With Phase 1 improvements)
```rust
// ~50 lines total including routes
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // One line to initialize everything from config
    initialize_from_config("auth.toml").await?;

    let app = Route::new()
        .at("/login", post(login))
        .at("/protected", get(protected));

    let state = PoemAppState::get();
    let listener = TcpListener::bind("0.0.0.0:3000");
    Server::new(listener).run(app).await?;

    Ok(())
}
```

---

## Testing Strategy

1. Update `examples/poem_example` to use new Phase 1 APIs
2. Verify it compiles and runs
3. Create `auth.toml` configuration file
4. Test database initialization from config
5. Test user creation from config
6. Verify app state is properly initialized and accessible in handlers

---

## Success Criteria

✅ Phase 1 Complete when:
- [ ] PoemAppState compiles and is usable
- [ ] AuthConfig loads from TOML files
- [ ] initialize_from_config() creates database and users
- [ ] poem_example reduces to <75 lines (from ~200)
- [ ] poem_example compiles and runs successfully
- [ ] Configuration file works with environment variable overrides
- [ ] All examples still work with the new setup

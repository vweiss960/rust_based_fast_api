/// Quick-start initialization for poem_auth
///
/// Provides a single function to initialize the entire authentication system
/// from a TOML configuration file, handling database creation, user setup,
/// and component initialization.

use crate::config::AuthConfig;
use crate::db::sqlite::SqliteUserDb;
use crate::db::{UserDatabase, UserRecord};
use crate::password::hash_password;
use crate::providers::LocalAuthProvider;
use crate::jwt::JwtValidator;
use crate::poem_integration::PoemAppState;

/// Initialize authentication system from configuration file
///
/// This function performs the following steps:
/// 1. Loads configuration from TOML file
/// 2. Validates configuration
/// 3. Creates/opens SQLite database
/// 4. Creates users from configuration (if they don't exist)
/// 5. Initializes LocalAuthProvider and JwtValidator
/// 6. Sets up global PoemAppState
///
/// # Arguments
///
/// * `config_path` - Path to TOML configuration file
///
/// # Errors
///
/// Returns error if any step fails (file not found, invalid config, database error, etc.)
///
/// # Example
///
/// ```ignore
/// use poem_auth::quick_start::initialize_from_config;
/// use poem::{Route, Server, get, post, handler, listener::TcpListener};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Initialize everything from config file
///     initialize_from_config("auth.toml").await?;
///
///     // Now use PoemAppState in handlers
///     let app = Route::new()
///         .at("/login", post(login))
///         .at("/protected", get(protected));
///
///     let listener = TcpListener::bind("0.0.0.0:3000");
///     Server::new(listener).run(app).await?;
///
///     Ok(())
/// }
/// ```
pub async fn initialize_from_config(
    config_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load and validate config
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
                println!(
                    "  ✓ Created: {} with groups {:?}",
                    user_config.username, user_config.groups
                );
            }
        }
    }
    println!();

    // Create auth components
    println!("Step 3: Create authentication components");
    let provider = std::sync::Arc::new(LocalAuthProvider::new(db));
    let jwt = std::sync::Arc::new(JwtValidator::new(&config.jwt.secret)?);
    println!("✓ LocalAuthProvider created");
    println!("✓ JwtValidator created\n");

    // Initialize global state
    let app_state = PoemAppState {
        provider,
        jwt,
    };
    app_state.init().map_err(|_| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to initialize PoemAppState - already initialized"
        )) as Box<dyn std::error::Error>
    })?;

    // Print summary
    println!("✅ Authentication system initialized successfully!");
    println!("\nConfiguration:");
    println!("  Database: {}", config.database.path);
    println!(
        "  JWT Secret: {}...{}",
        &config.jwt.secret[..8],
        &config.jwt.secret[config.jwt.secret.len() - 4..]
    );
    println!("  Token Expiration: {} hours", config.jwt.expiration_hours);
    println!("  Users: {}", config.users.len());

    if let Some(server) = &config.server {
        println!(
            "\nServer: http://{}:{}",
            server.host, server.port
        );
    }
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_initialize_from_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("auth.toml");

        let config_content = r#"
[database]
path = "test.db"
auto_create = true

[jwt]
secret = "my-super-secret-key-should-be-at-least-16-chars"
expiration_hours = 24

[[users]]
username = "alice"
password = "password123"
groups = ["users"]
enabled = true
"#;

        fs::write(&config_path, config_content).unwrap();

        // Note: This test would need cleanup of global state
        // For now, we just test that it doesn't panic
        let result = initialize_from_config(config_path.to_str().unwrap()).await;
        assert!(result.is_ok());
    }
}

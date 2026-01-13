use poem_auth::api::types::LoginRequest;
use poem_auth::prelude::*;
use poem_auth::db::sqlite::SqliteUserDb;
use poem_auth::providers::LocalAuthProvider;
use poem_auth::jwt::JwtValidator;
use poem_auth::db::UserRecord;
use poem_auth::password::hash_password;

/// poem_auth REST API Example
///
/// This example demonstrates how to build a REST API that uses poem_auth
/// It shows:
/// - Database initialization
/// - User creation
/// - Authentication
/// - JWT token generation
/// - Token validation

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== poem_auth REST API Example ===\n");

    // Step 1: Initialize database
    println!("Step 1: Initialize database");
    let db = SqliteUserDb::new("api-example.db").await?;
    println!("✓ Database initialized\n");

    // Step 2: Create test users
    println!("Step 2: Create test users for REST API");
    for (username, password) in &[("alice", "password123"), ("bob", "secret456"), ("charlie", "mysecret")] {
        if db.get_user(username).await.is_err() {
            let hash = hash_password(password)?;
            let user = UserRecord::new(username, &hash)
                .with_groups(vec!["users"]);
            db.create_user(user).await?;
            println!("✓ Created user: {}", username);
        }
    }
    println!();

    // Step 3: Create auth components
    println!("Step 3: Create authentication components");
    let provider = LocalAuthProvider::new(db);
    let jwt = JwtValidator::new("my-super-secret-key-should-be-at-least-16-chars")?;
    println!("✓ LocalAuthProvider created");
    println!("✓ JwtValidator created\n");

    // Step 4: Simulate REST API requests
    println!("Step 4: Simulate REST API Requests\n");

    println!("=== GET / (Health Check) ===");
    println!("Response: 200 OK");
    println!("  {{\"status\": \"ok\"}}\n");

    println!("=== POST /login ===");
    println!("Request Body: {{\"username\": \"alice\", \"password\": \"password123\"}}");

    match provider.authenticate("alice", "password123").await {
        Ok(claims) => {
            match jwt.generate_token(&claims) {
                Ok(token_data) => {
                    println!("Response: 200 OK");
                    println!("  {{");
                    println!("    \"token\": \"{}\",", &token_data.token[..50]);
                    println!("    \"token_type\": \"Bearer\",");
                    println!("    \"expires_in\": {},", claims.exp - claims.iat);
                    println!("    \"claims\": {{");
                    println!("      \"sub\": \"{}\",", claims.sub);
                    println!("      \"groups\": {:?}", claims.groups);
                    println!("    }}");
                    println!("  }}\n");

                    // Step 5: Demonstrate token validation
                    println!("=== GET /profile (Protected Route) ===");
                    println!("Authorization: Bearer {}", &token_data.token[..50]);

                    match jwt.verify_token(&token_data.token) {
                        Ok(decoded_claims) => {
                            println!("Response: 200 OK");
                            println!("  {{");
                            println!("    \"username\": \"{}\",", decoded_claims.sub);
                            println!("    \"groups\": {:?},", decoded_claims.groups);
                            println!("    \"message\": \"Access granted\"");
                            println!("  }}\n");
                        }
                        Err(e) => println!("Response: 401 Unauthorized - {}\n", e),
                    }
                }
                Err(e) => {
                    println!("Response: 500 Internal Server Error");
                    println!("  {{\"error\": \"Failed to generate token: {}\"}}\n", e);
                }
            }
        }
        Err(e) => {
            println!("Response: 401 Unauthorized");
            println!("  {{\"error\": \"invalid_credentials\", \"message\": \"{}\"}}\n", e);
        }
    }

    // Step 6: Test invalid credentials
    println!("=== POST /login (Invalid Credentials) ===");
    println!("Request Body: {{\"username\": \"alice\", \"password\": \"wrongpassword\"}}");
    match provider.authenticate("alice", "wrongpassword").await {
        Ok(_) => println!("ERROR: Should have failed!"),
        Err(_) => {
            println!("Response: 401 Unauthorized");
            println!("  {{\"error\": \"invalid_credentials\", \"message\": \"Invalid username or password\"}}\n");
        }
    }

    println!("=== Example Complete ===");
    println!("\nKey Concepts:");
    println!("  - LocalAuthProvider.authenticate() validates credentials");
    println!("  - JwtValidator.generate_token() creates signed JWT tokens");
    println!("  - JwtValidator.verify_token() validates token signatures");
    println!("  - Tokens can be used to authenticate subsequent requests");

    println!("\nTo build a full REST API with Poem:");
    println!("  1. Use poem::Route to define endpoints");
    println!("  2. Pass AppState with provider/jwt to handlers");
    println!("  3. Use poem_auth middleware to extract UserClaims from headers");
    println!("  4. Protect routes that require authentication");

    println!("\nSee INTEGRATION_GUIDE.md for full Poem web server example!");

    Ok(())
}

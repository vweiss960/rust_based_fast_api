use poem_auth::prelude::*;
use poem_auth::db::sqlite::SqliteUserDb;
use poem_auth::providers::LocalAuthProvider;
use poem_auth::jwt::JwtValidator;
use poem_auth::db::UserRecord;
use poem_auth::password::hash_password;

/// Basic example demonstrating poem_auth core functionality
/// This shows how to:
/// 1. Initialize a database
/// 2. Create users
/// 3. Authenticate with credentials
/// 4. Generate and validate JWT tokens

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== poem_auth Basic Example ===\n");

    // Step 1: Set up the database
    println!("Step 1: Initialize database");
    let db = SqliteUserDb::new("basic-auth.db").await?;
    println!("✓ Database initialized\n");

    // Step 2: Create test user
    println!("Step 2: Create test user");
    match db.get_user("alice").await {
        Err(_) => {
            let password_hash = hash_password("password123")?;
            let user = UserRecord::new("alice", &password_hash)
                .with_groups(vec!["users", "developers"]);
            db.create_user(user).await?;
            println!("✓ Test user 'alice' created");
            println!("  - Username: alice");
            println!("  - Password: password123");
            println!("  - Groups: [users, developers]\n");
        }
        Ok(_) => {
            println!("✓ Test user 'alice' already exists\n");
        }
    }

    // Step 3: Create authentication provider
    println!("Step 3: Create LocalAuthProvider");
    let provider = LocalAuthProvider::new(db.clone());
    println!("✓ LocalAuthProvider created\n");

    // Step 4: Create JWT validator
    println!("Step 4: Create JwtValidator");
    let jwt = JwtValidator::new("my-super-secret-key-should-be-at-least-16-chars")?;
    println!("✓ JwtValidator created\n");

    // Step 5: Authenticate user
    println!("Step 5: Authenticate user");
    println!("Authenticating alice with password123...\n");
    match provider.authenticate("alice", "password123").await {
        Ok(claims) => {
            println!("✓ Authentication successful!");
            println!("  - Subject: {}", claims.sub);
            println!("  - Provider: {}", claims.provider);
            println!("  - Groups: {:?}", claims.groups);
            println!("  - Issued: {}", claims.iat);
            println!("  - Expires: {}\n", claims.exp);

            // Step 6: Generate JWT token
            println!("Step 6: Generate JWT token");
            let token = jwt.generate_token(&claims)?;
            println!("✓ Token generated:");
            println!("  {}", token.token);
            println!("  ({}bytes)\n", token.token.len());

            // Step 7: Validate token
            println!("Step 7: Validate JWT token");
            let decoded = jwt.verify_token(&token.token)?;
            println!("✓ Token validated successfully!");
            println!("  - Subject: {}", decoded.sub);
            println!("  - Groups: {:?}\n", decoded.groups);
        }
        Err(e) => {
            println!("✗ Authentication failed: {}\n", e);
            return Err(Box::new(e));
        }
    }

    // Step 8: Test invalid credentials
    println!("Step 8: Test invalid credentials");
    println!("Authenticating alice with wrong password...\n");
    match provider.authenticate("alice", "wrongpassword").await {
        Ok(_) => println!("✗ ERROR: Should have failed!"),
        Err(_) => println!("✓ Correctly rejected invalid password\n"),
    }

    println!("=== Example Complete ===");
    println!("\nKey concepts demonstrated:");
    println!("  - SqliteUserDb for user storage");
    println!("  - LocalAuthProvider for credential verification");
    println!("  - JwtValidator for token generation/validation");
    println!("  - UserClaims for authenticated user information");
    println!("\nTo use in Poem web app:");
    println!("  1. Initialize these components in your main()");
    println!("  2. Add poem_auth middleware to extract claims");
    println!("  3. Use claims in your route handlers");
    println!("\nSee GETTING_STARTED.md for full Poem integration!");

    Ok::<_, Box<dyn std::error::Error>>(())
}

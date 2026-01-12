use poem_auth::prelude::*;
use poem_auth::db::sqlite::SqliteUserDb;
use poem_auth::providers::LocalAuthProvider;
use poem_auth::jwt::JwtValidator;
use poem_auth::db::UserRecord;
use poem_auth::password::hash_password;
use poem::{listener::TcpListener, App};

async fn health_check() -> String {
    "OK".to_string()
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing authentication system...");

    // Step 1: Set up the database
    let db = SqliteUserDb::new("users.db").await?;
    println!("✓ Database initialized");

    // Step 2: Create test user
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
    let _provider = LocalAuthProvider::new(db.clone());

    // Step 4: Create JWT validator
    let _jwt = JwtValidator::new("my-super-secret-key-should-be-at-least-16-chars")?;

    // Step 5: Build the Poem app with routes
    let app = App::new()
        .at("/health", poem::get(health_check))
        .at("/public", poem::get(public_handler))
        .at("/protected", poem::get(protected_handler));

    // Step 6: Run the server
    let addr = "127.0.0.1:3000";
    println!("🚀 Server running at http://{}", addr);
    println!("\nTry these endpoints:");
    println!("  GET http://{}/health", addr);
    println!("  GET http://{}/public", addr);
    println!("  GET http://{}/protected (requires bearer token)", addr);

    app.run(TcpListener::bind(addr)).await?;
    Ok(())
}

use poem::{listener::TcpListener, App, get, post, web::Json};
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
    println!("Initializing authentication system with login endpoint...");

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

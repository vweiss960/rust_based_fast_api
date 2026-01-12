use poem::{listener::TcpListener, App, get, post, web::Json, http::StatusCode};
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
    println!("Initializing RBAC authentication system...");

    let db = SqliteUserDb::new("users.db").await?;
    println!("✓ Database initialized");

    // Create test users with different roles
    create_test_users(&db).await?;

    let provider = LocalAuthProvider::new(db.clone());
    let jwt = JwtValidator::new("my-super-secret-key-should-be-at-least-16-chars")?;

    let provider = Arc::new(provider);
    let jwt = Arc::new(jwt);

    let app = App::new()
        .at("/health", get(health_check))
        .at("/login", post({
            let provider = provider.clone();
            let jwt = jwt.clone();
            move |req| login(req, provider.clone(), jwt.clone())
        }))
        .at("/user", get(user_handler))
        .at("/admin", get(admin_handler))
        .at("/developer", get(developer_handler));

    let addr = "127.0.0.1:3000";
    println!("🚀 Server running at http://{}", addr);
    println!("\nTest users:");
    println!("  alice (password: password123) - groups: users, developers");
    println!("  bob (password: password123) - groups: users, admins");
    println!("\nLogin and get token, then access /user, /admin, or /developer");

    app.run(TcpListener::bind(addr)).await?;
    Ok(())
}

async fn create_test_users(db: &SqliteUserDb) -> Result<(), Box<dyn std::error::Error>> {
    // Create alice (developer)
    if db.get_user("alice").await.is_err() {
        let hash = hash_password("password123")?;
        let user = UserRecord::new("alice", &hash)
            .with_groups(vec!["users", "developers"]);
        db.create_user(user).await?;
        println!("✓ Test user 'alice' created (developer)");
    }

    // Create bob (admin)
    if db.get_user("bob").await.is_err() {
        let hash = hash_password("password123")?;
        let user = UserRecord::new("bob", &hash)
            .with_groups(vec!["users", "admins"]);
        db.create_user(user).await?;
        println!("✓ Test user 'bob' created (admin)");
    }

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn login(
    req: Json<LoginRequest>,
    provider: Arc<LocalAuthProvider>,
    jwt: Arc<JwtValidator>,
) -> Result<Json<LoginResponse>, Json<ErrorResponse>> {
    let claims = provider.authenticate(&req.username, &req.password)
        .await
        .map_err(|_| Json(ErrorResponse::invalid_credentials()))?;

    let token = jwt.encode(&claims)
        .map_err(|_| Json(ErrorResponse::new("error", "Failed to generate token")))?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: claims.exp - claims.iat,
        claims: UserClaimsResponse::from_claims(claims),
    }))
}

async fn user_handler(claims: UserClaims) -> Result<String, (StatusCode, String)> {
    // All authenticated users can access this
    Ok(format!("Hello {}! You are authenticated.", claims.sub))
}

async fn admin_handler(claims: UserClaims) -> Result<String, (StatusCode, String)> {
    // Only admins can access this
    if claims.has_group("admins") {
        Ok(format!("Welcome admin {}! You have full access.", claims.sub))
    } else {
        Err((StatusCode::FORBIDDEN, "Admin access required".to_string()))
    }
}

async fn developer_handler(claims: UserClaims) -> Result<String, (StatusCode, String)> {
    // Only developers can access this
    if claims.has_group("developers") {
        Ok(format!("Welcome developer {}! Access to developer tools.", claims.sub))
    } else {
        Err((StatusCode::FORBIDDEN, "Developer access required".to_string()))
    }
}

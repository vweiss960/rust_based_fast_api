use poem::{
    Route, Server, get, post, handler, listener::TcpListener, web::Path, web::Json,
    http::StatusCode, Response, IntoResponse
};
use poem_auth::{
    initialize_from_config, PoemAppState, AuthProvider, UserClaims,
    api::types::LoginRequest,
    poem_integration::guards::{AuthGuard, HasGroup, HasAnyGroup},
};
use serde_json::json;

/// Public endpoint - anyone can access
#[handler]
fn hello(Path(name): Path<String>) -> String {
    format!("hello: {}", name)
}

/// Health check endpoint
#[handler]
fn health() -> String {
    json!({"status": "ok"}).to_string()
}

/// Login endpoint - returns JWT token or error response
#[handler]
async fn login(Json(req): Json<LoginRequest>) -> Response {
    let state = PoemAppState::get();
    match state.provider.authenticate(&req.username, &req.password).await {
        Ok(claims) => {
            match state.jwt.generate_token(&claims) {
                Ok(token_data) => {
                    (StatusCode::OK, Json(json!({
                        "token": token_data.token,
                        "token_type": "Bearer",
                        "expires_in": claims.exp - claims.iat,
                        "user": {
                            "username": claims.sub,
                            "groups": claims.groups
                        }
                    }))).into_response()
                }
                Err(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                        "error": "Failed to generate token"
                    }))).into_response()
                }
            }
        }
        Err(_) => {
            (StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Invalid credentials"
            }))).into_response()
        }
    }
}

/// Protected endpoint with automatic UserClaims extraction via FromRequest
///
/// Phase 2 Feature: No manual token extraction needed!
/// The `claims: UserClaims` parameter is automatically extracted and validated
/// from the Authorization header using Poem's FromRequest trait.
#[handler]
async fn protected(claims: UserClaims) -> Response {
    (StatusCode::OK, Json(json!({
        "message": "Access granted",
        "username": claims.sub,
        "groups": claims.groups,
        "expires_in": claims.exp - claims.iat
    }))).into_response()
}

/// Admin-only endpoint with guard-based authorization
///
/// Phase 2 Feature: Demonstrates guard-based permission checking.
/// Uses HasGroup guard to ensure user has 'admins' group.
#[handler]
async fn admin_endpoint(claims: UserClaims) -> Response {
    let guard = HasGroup("admins".to_string());

    if guard.check(&claims) {
        (StatusCode::OK, Json(json!({
            "message": "Admin access granted",
            "username": claims.sub,
            "admin_group": "admins"
        }))).into_response()
    } else {
        (StatusCode::FORBIDDEN, Json(json!({
            "error": "This endpoint requires 'admins' group membership"
        }))).into_response()
    }
}

/// Moderator OR admin endpoint
///
/// Phase 2 Feature: Demonstrates composable guards with OR logic.
/// User is allowed if they have either 'moderators' or 'admins' group.
#[handler]
async fn moderator_endpoint(claims: UserClaims) -> Response {
    let guard = HasAnyGroup(vec!["admins".to_string(), "moderators".to_string()]);

    if guard.check(&claims) {
        (StatusCode::OK, Json(json!({
            "message": "Moderator access granted",
            "username": claims.sub,
            "role": if claims.has_group("admins") { "admin" } else { "moderator" }
        }))).into_response()
    } else {
        (StatusCode::FORBIDDEN, Json(json!({
            "error": "This endpoint requires 'admins' or 'moderators' group membership"
        }))).into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== poem + poem_auth Example ===\n");

    // Initialize everything from config file in ONE LINE!
    initialize_from_config("auth.toml").await?;

    // Build app routes
    println!("Step 4: Building Poem app with routes\n");
    let app = Route::new()
        .at("/", get(health))
        .at("/hello/:name", get(hello))
        .at("/login", post(login))
        .at("/protected", get(protected))
        .at("/admin", get(admin_endpoint))
        .at("/moderator", get(moderator_endpoint));

    let addr = "0.0.0.0:3000";
    println!("🚀 Server running at http://{}\n", addr);
    println!("Available endpoints:");
    println!("  GET  http://localhost:3000/                    - Health check");
    println!("  GET  http://localhost:3000/hello/:name         - Public greeting");
    println!("  POST http://localhost:3000/login               - Login to get token");
    println!("  GET  http://localhost:3000/protected           - Protected endpoint (Phase 2: auto extraction)\n");
    println!("  GET  http://localhost:3000/admin               - Admin-only endpoint (Phase 2: guard checks)\n");
    println!("  GET  http://localhost:3000/moderator           - Moderator/Admin endpoint (Phase 2: OR guard)\n");

    println!("Example requests:");
    println!("  # Health check");
    println!("  curl http://localhost:3000/\n");

    println!("  # Get greeting");
    println!("  curl http://localhost:3000/hello/World\n");

    println!("  # Login (alice has users + admins groups)");
    println!("  curl -X POST http://localhost:3000/login \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"username\":\"alice\",\"password\":\"password123\"}}'\n");

    println!("  # Login (bob has only users group)");
    println!("  curl -X POST http://localhost:3000/login \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"username\":\"bob\",\"password\":\"secret456\"}}'\n");

    println!("  # Access protected endpoint (replace TOKEN with token from login)");
    println!("  curl -H 'Authorization: Bearer TOKEN' http://localhost:3000/protected\n");

    println!("  # Access admin endpoint (only works with admin token)");
    println!("  curl -H 'Authorization: Bearer TOKEN' http://localhost:3000/admin\n");

    println!("  # Access moderator endpoint (works with admin or moderator token)");
    println!("  curl -H 'Authorization: Bearer TOKEN' http://localhost:3000/moderator\n");

    let listener = TcpListener::bind(addr);
    Server::new(listener).run(app).await?;

    Ok(())
}

use poem::{
    Route, Server, get, post, handler, listener::TcpListener, web::Path, web::Json,
    http::StatusCode, Response, IntoResponse
};
use poem_auth::{
    initialize_from_config, PoemAppState, AuthProvider, UserClaims,
    api::types::LoginRequest,
    poem_integration::guards::{AuthGuard, HasGroup, HasAnyGroup},
    LoginResponseBuilder,
    require_group, require_any_groups, require_all_groups,
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
///
/// Simplified using LoginResponseBuilder for minimal boilerplate.
#[handler]
async fn login(Json(req): Json<LoginRequest>) -> Response {
    let state = PoemAppState::get();
    match state.provider.authenticate(&req.username, &req.password).await {
        Ok(claims) => {
            match state.jwt.generate_token(&claims) {
                Ok(token_data) => LoginResponseBuilder::success(&claims, &token_data),
                Err(_) => LoginResponseBuilder::token_generation_failed(),
            }
        }
        Err(_) => LoginResponseBuilder::invalid_credentials(),
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

/// Phase 2B: Admin endpoint using macro (zero boilerplate!)
///
/// This macro automatically checks group membership and returns 403 if denied.
/// No manual guard instantiation needed!
#[require_group("admins")]
#[handler]
async fn admin_macro(claims: UserClaims) -> Response {
    (StatusCode::OK, Json(json!({
        "message": "Admin access granted via macro!",
        "username": claims.sub
    }))).into_response()
}

/// Phase 2B: Moderator endpoint using macro with OR logic
///
/// The require_any_groups macro allows access if user has ANY of the groups.
/// Automatically applies HasAnyGroup guard and returns 403 if all denied.
#[require_any_groups("admins", "moderators")]
#[handler]
async fn moderator_macro(claims: UserClaims) -> Response {
    (StatusCode::OK, Json(json!({
        "message": "Moderator/Admin access granted via macro!",
        "username": claims.sub,
        "role": if claims.has_group("admins") { "admin" } else { "moderator" }
    }))).into_response()
}

/// Phase 2B: Verified developer endpoint using macro with AND logic
///
/// The require_all_groups macro requires user to have ALL specified groups.
/// Automatically applies HasAllGroups guard and returns 403 if any group missing.
#[require_all_groups("developers", "verified")]
#[handler]
async fn verified_dev_macro(claims: UserClaims) -> Response {
    (StatusCode::OK, Json(json!({
        "message": "Verified developer area (macro-protected)!",
        "username": claims.sub,
        "developer": claims.has_group("developers"),
        "verified": claims.has_group("verified")
    }))).into_response()
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
        .at("/moderator", get(moderator_endpoint))
        // Phase 2B: Macro-based endpoints (zero boilerplate!)
        .at("/admin/macro", get(admin_macro))
        .at("/moderator/macro", get(moderator_macro))
        .at("/dev/macro", get(verified_dev_macro));

    // Get server configuration from global state
    let state = PoemAppState::get();
    state.validate_listener_config()?;

    let addr = state.listener_addr();
    let protocol = if state.tls_enabled() { "https" } else { "http" };

    println!("Step 5: Starting server\n");
    println!("🚀 Server running at {}://{}\n", protocol, addr);
    println!("Available endpoints:");
    println!("  GET  {}://{}/                    - Health check", protocol, addr);
    println!("  GET  {}://{}/hello/:name         - Public greeting", protocol, addr);
    println!("  POST {}://{}/login               - Login to get token", protocol, addr);
    println!("  GET  {}://{}/protected           - Protected endpoint (Phase 2: auto extraction)", protocol, addr);
    println!("  GET  {}://{}/admin               - Admin-only endpoint (Phase 2: guard checks)", protocol, addr);
    println!("  GET  {}://{}/moderator           - Moderator/Admin endpoint (Phase 2: OR guard)", protocol, addr);
    println!();
    println!("  Phase 2B - Macro-based endpoints (zero boilerplate!):");
    println!("  GET  {}://{}/admin/macro         - Admin-only (macro: #[require_group])", protocol, addr);
    println!("  GET  {}://{}/moderator/macro     - Moderator/Admin (macro: #[require_any_groups])", protocol, addr);
    println!("  GET  {}://{}/dev/macro           - Verified developers (macro: #[require_all_groups])", protocol, addr);
    println!();

    println!("Example requests:");
    println!("  # Health check");
    println!("  curl {}://{}/\n", protocol, addr);

    println!("  # Get greeting");
    println!("  curl {}://{}/hello/World\n", protocol, addr);

    println!("  # Login (alice has users + admins groups)");
    println!("  curl -X POST {}://{}/login \\", protocol, addr);
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"username\":\"alice\",\"password\":\"password123\"}}'\n");

    println!("  # Login (bob has only users group)");
    println!("  curl -X POST {}://{}/login \\", protocol, addr);
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"username\":\"bob\",\"password\":\"secret456\"}}'\n");

    println!("  # Access protected endpoint (replace TOKEN with token from login)");
    println!("  curl -H 'Authorization: Bearer TOKEN' {}://{}/protected\n", protocol, addr);

    println!("  # Access admin endpoint (only works with admin token)");
    println!("  curl -H 'Authorization: Bearer TOKEN' {}://{}/admin\n", protocol, addr);

    println!("  # Access moderator endpoint (works with admin or moderator token)");
    println!("  curl -H 'Authorization: Bearer TOKEN' {}://{}/moderator\n", protocol, addr);

    let listener = TcpListener::bind(&addr);
    Server::new(listener).run(app).await?;

    Ok(())
}

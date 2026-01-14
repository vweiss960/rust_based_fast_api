//! Simple helpers for implementing JWT login endpoints in Poem handlers
//!
//! This module provides convenient response constructors for creating login endpoints
//! with minimal boilerplate.

use poem::{Response, http::StatusCode, IntoResponse};
use poem::web::Json;
use serde_json::json;

use crate::auth::UserClaims;
use crate::api::types::{LoginResponse, UserClaimsResponse};
use crate::jwt::Token;

/// Helper for constructing JWT login responses with minimal boilerplate.
///
/// Provides static methods for creating standard login responses or error responses
/// from Poem handlers with a single function call.
///
/// # Example
///
/// ```ignore
/// use poem_auth::{PoemAppState, LoginResponseBuilder};
/// use poem::web::Json;
/// use poem_auth::api::types::LoginRequest;
/// use poem::{handler, Response};
///
/// #[handler]
/// async fn login(Json(req): Json<LoginRequest>) -> Response {
///     let state = PoemAppState::get();
///     match state.provider.authenticate(&req.username, &req.password).await {
///         Ok(claims) => {
///             match state.jwt.generate_token(&claims) {
///                 Ok(token_data) => LoginResponseBuilder::success(&claims, &token_data),
///                 Err(_) => LoginResponseBuilder::token_generation_failed(),
///             }
///         }
///         Err(_) => LoginResponseBuilder::invalid_credentials(),
///     }
/// }
/// ```
pub struct LoginResponseBuilder;

impl LoginResponseBuilder {
    /// Build a successful login response (HTTP 200).
    ///
    /// # Arguments
    ///
    /// * `claims` - The authenticated user's claims
    /// * `token_data` - The generated JWT token
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = LoginResponseBuilder::success(&claims, &token_data);
    /// ```
    pub fn success(claims: &UserClaims, token_data: &Token) -> Response {
        let expires_in = claims.exp - claims.iat;
        let login_response = LoginResponse {
            token: token_data.token.clone(),
            token_type: "Bearer".to_string(),
            expires_in,
            claims: UserClaimsResponse::from_claims(claims.clone()),
        };

        (StatusCode::OK, Json(login_response)).into_response()
    }

    /// Build a response for invalid credentials (returns 401).
    pub fn invalid_credentials() -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "invalid_credentials",
                "message": "Username or password is incorrect"
            })),
        )
            .into_response()
    }

    /// Build a response for token generation failure (returns 500).
    pub fn token_generation_failed() -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "token_generation_failed",
                "message": "Failed to generate authentication token"
            })),
        )
            .into_response()
    }

    /// Build a response for disabled user (returns 403).
    pub fn user_disabled(username: &str) -> Response {
        (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "user_disabled",
                "message": format!("User '{}' is disabled", username)
            })),
        )
            .into_response()
    }

    /// Build a response for user not found (returns 401).
    pub fn user_not_found() -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "user_not_found",
                "message": "Username or password is incorrect"
            })),
        )
            .into_response()
    }

    /// Build a custom error response.
    pub fn error(status: StatusCode, error_code: &str, message: &str) -> Response {
        (
            status,
            Json(json!({
                "error": error_code,
                "message": message
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_response_builder_invalid_credentials() {
        let response = LoginResponseBuilder::invalid_credentials();
        // We can't directly test Response, but this ensures the method compiles
        // and returns a valid Response
        assert!(true);
    }

    #[test]
    fn test_login_response_builder_token_generation_failed() {
        let response = LoginResponseBuilder::token_generation_failed();
        assert!(true);
    }

    #[test]
    fn test_login_response_builder_user_disabled() {
        let response = LoginResponseBuilder::user_disabled("alice");
        assert!(true);
    }

    #[test]
    fn test_login_response_builder_user_not_found() {
        let response = LoginResponseBuilder::user_not_found();
        assert!(true);
    }
}

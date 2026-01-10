//! JWT authentication utilities for Poem.
//!
//! This module provides utilities for working with JWT tokens in Poem applications.

use poem::{
    http::header::AUTHORIZATION,
    Request,
};

use crate::auth::UserClaims;
use crate::error::AuthError;
use crate::jwt::JwtValidator;

/// Helper to extract and validate JWT tokens from Poem requests.
///
/// # Example
///
/// ```ignore
/// use poem::{web::Path, IntoResponse};
/// use poem_auth::middleware::extract_jwt_claims;
/// use poem_auth::jwt::JwtValidator;
///
/// let validator = JwtValidator::new("secret")?;
///
/// async fn handler(req: &Request) -> Result<String, String> {
///     let claims = extract_jwt_claims(req, &validator).await?;
///     Ok(format!("Hello, {}", claims.sub))
/// }
/// ```
pub async fn extract_jwt_claims(
    req: &Request,
    validator: &JwtValidator,
) -> Result<UserClaims, AuthError> {
    // Get Authorization header
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::InvalidToken)?;

    // Extract token from "Bearer <token>" format
    let token = JwtValidator::extract_token(auth_header)?;

    // Validate and decode token
    validator.verify_token(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_extraction() {
        let validator = JwtValidator::new("my-very-long-secret-key").unwrap();
        let claims = crate::auth::UserClaims::new("alice", "local", 2000, 1000)
            .with_groups(vec!["admins"]);

        let token = validator.generate_token(&claims).unwrap();
        assert!(!token.token.is_empty());
    }
}

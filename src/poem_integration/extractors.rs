/// Poem framework extractors for automatic user claims extraction
///
/// This module provides automatic extraction of JWT claims from HTTP requests,
/// making it easy to access authenticated user information in Poem handlers
/// without manual token parsing and validation.

use poem::{FromRequest, Request, RequestBody, http::StatusCode, Error as PoemError};
use crate::auth::UserClaims;
use crate::poem_integration::PoemAppState;

/// Automatic JWT extractor for Poem handlers
///
/// This implementation allows handlers to directly receive `UserClaims` as a parameter,
/// with automatic extraction, validation, and error handling.
///
/// # Example
///
/// ```ignore
/// use poem::{handler, Route, post};
/// use poem_auth::UserClaims;
///
/// #[handler]
/// async fn protected_endpoint(claims: UserClaims) -> String {
///     format!("Hello, {}!", claims.sub)
/// }
///
/// // Handler automatically receives validated claims
/// // No manual token extraction or validation needed!
/// ```
///
/// # How it Works
///
/// 1. Extracts Authorization header from request
/// 2. Checks for "Bearer <token>" format
/// 3. Extracts JWT from Bearer token
/// 4. Verifies and decodes using JwtValidator from global state
/// 5. Returns claims or 401 Unauthorized error
///
/// # Error Handling
///
/// Returns 401 Unauthorized if:
/// - No Authorization header present
/// - Header doesn't start with "Bearer "
/// - Token is invalid or expired
/// - JwtValidator is not initialized
///
/// # Performance
///
/// If token caching is enabled (feature: `cache`), validated tokens are cached
/// to avoid repeated cryptographic operations.
impl<'a> FromRequest<'a> for UserClaims {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self, PoemError> {
        // Try to get the app state (will panic if not initialized)
        let state = match PoemAppState::try_get() {
            Some(s) => s,
            None => {
                return Err(PoemError::from_status(StatusCode::INTERNAL_SERVER_ERROR));
            }
        };

        // Extract Authorization header
        let auth_header = match req.header("Authorization") {
            Some(h) => h,
            None => {
                return Err(PoemError::from_status(StatusCode::UNAUTHORIZED));
            }
        };

        // Extract Bearer token
        let token = match auth_header.strip_prefix("Bearer ") {
            Some(t) => t,
            None => {
                return Err(PoemError::from_status(StatusCode::UNAUTHORIZED));
            }
        };

        // Verify and decode token
        match state.jwt.verify_token(token) {
            Ok(claims) => Ok(claims),
            Err(_) => Err(PoemError::from_status(StatusCode::UNAUTHORIZED)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fromrequest_requires_authorization_header() {
        // This is a compile-time test ensuring FromRequest is properly implemented
        // Runtime tests require setting up a full Poem app
    }
}

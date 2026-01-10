//! Request and response types for authentication API endpoints.

use serde::{Deserialize, Serialize};
use crate::auth::UserClaims;

/// Login request payload.
///
/// Used to authenticate a user with username and password,
/// optionally specifying which authentication provider to use.
///
/// # Example
///
/// ```ignore
/// let login_request = LoginRequest {
///     username: "alice".to_string(),
///     password: "password123".to_string(),
///     provider: Some("local".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    /// The username to authenticate
    pub username: String,
    /// The plaintext password
    pub password: String,
    /// Optional: which auth provider to use ("local", "ldap", etc.)
    /// If None, tries default provider
    #[serde(default)]
    pub provider: Option<String>,
}

/// Successful login response.
///
/// Contains the JWT token and metadata about the token's validity.
///
/// # Example
///
/// ```ignore
/// let response = LoginResponse {
///     token: "eyJhbGc...".to_string(),
///     token_type: "Bearer".to_string(),
///     expires_in: 86400,
///     claims: claims,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    /// The JWT token string
    pub token: String,
    /// Token type (always "Bearer" for JWT)
    pub token_type: String,
    /// Seconds until the token expires
    pub expires_in: i64,
    /// The decoded claims (user info)
    pub claims: UserClaimsResponse,
}

/// Simplified user claims for API responses.
///
/// Exposes user information without sensitive fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaimsResponse {
    /// The authenticated username
    pub sub: String,
    /// The authentication provider used
    pub provider: String,
    /// User's groups/roles
    pub groups: Vec<String>,
    /// Token expiration time (Unix timestamp)
    pub exp: i64,
    /// Token issued at time (Unix timestamp)
    pub iat: i64,
}

impl UserClaimsResponse {
    /// Create from UserClaims struct.
    pub fn from_claims(claims: UserClaims) -> Self {
        Self {
            sub: claims.sub,
            provider: claims.provider,
            groups: claims.groups,
            exp: claims.exp,
            iat: claims.iat,
        }
    }
}

/// Create user request payload.
///
/// Used by administrators to create new local users.
/// Requires master authentication.
///
/// # Example
///
/// ```ignore
/// let request = CreateUserRequest {
///     username: "bob".to_string(),
///     password: "secure-password".to_string(),
///     groups: vec!["users".to_string()],
///     enabled: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    /// The new username
    pub username: String,
    /// The new user's plaintext password (will be hashed)
    pub password: String,
    /// Initial groups/roles for the user
    #[serde(default)]
    pub groups: Vec<String>,
    /// Whether the user account is initially enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Update password request payload.
///
/// Used by users or administrators to change a user's password.
///
/// # Example
///
/// ```ignore
/// let request = UpdatePasswordRequest {
///     username: "alice".to_string(),
///     new_password: "new-secure-password".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePasswordRequest {
    /// The username whose password to update
    pub username: String,
    /// The new plaintext password (will be hashed)
    pub new_password: String,
}

/// User information response.
///
/// Public user information for list/get operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    /// The username
    pub username: String,
    /// Whether the account is enabled
    pub enabled: bool,
    /// User's groups/roles
    pub groups: Vec<String>,
    /// Account creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
}

/// API error response.
///
/// Standardized error response format for all API endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Machine-readable error code
    pub error: String,
    /// Human-readable error message
    pub message: String,
    /// Optional additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ErrorResponse {
    /// Create a new error response.
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    /// Create an error response with details.
    pub fn with_details(error: &str, message: &str, details: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            details: Some(details.to_string()),
        }
    }

    /// Invalid credentials error.
    pub fn invalid_credentials() -> Self {
        Self::new("invalid_credentials", "Username or password is incorrect")
    }

    /// User not found error.
    pub fn user_not_found(username: &str) -> Self {
        Self::new("user_not_found", &format!("User '{}' not found", username))
    }

    /// User disabled error.
    pub fn user_disabled(username: &str) -> Self {
        Self::new("user_disabled", &format!("User '{}' is disabled", username))
    }

    /// Unauthorized access error.
    pub fn unauthorized() -> Self {
        Self::new("unauthorized", "Authorization required")
    }

    /// Forbidden access error.
    pub fn forbidden(reason: &str) -> Self {
        Self::new("forbidden", reason)
    }
}

/// Helper function for default_true in serde.
fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_serialization() {
        let req = LoginRequest {
            username: "alice".to_string(),
            password: "pass123".to_string(),
            provider: Some("local".to_string()),
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: LoginRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.username, "alice");
        assert_eq!(deserialized.password, "pass123");
        assert_eq!(deserialized.provider, Some("local".to_string()));
    }

    #[test]
    fn test_user_claims_response_from_claims() {
        let claims = UserClaims::new("bob", "ldap", 2000, 1000)
            .with_groups(vec!["admins", "users"]);

        let response = UserClaimsResponse::from_claims(claims.clone());

        assert_eq!(response.sub, "bob");
        assert_eq!(response.provider, "ldap");
        assert_eq!(response.groups, vec!["admins", "users"]);
        assert_eq!(response.exp, 2000);
        assert_eq!(response.iat, 1000);
    }

    #[test]
    fn test_create_user_request_default_enabled() {
        let json = r#"{"username":"alice","password":"pass123"}"#;
        let req: CreateUserRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.username, "alice");
        assert_eq!(req.password, "pass123");
        assert_eq!(req.groups, Vec::<String>::new());
        assert_eq!(req.enabled, true);  // Should default to true
    }

    #[test]
    fn test_error_response_construction() {
        let error = ErrorResponse::new("test_error", "Test message");
        assert_eq!(error.error, "test_error");
        assert_eq!(error.message, "Test message");
        assert_eq!(error.details, None);
    }

    #[test]
    fn test_error_response_with_details() {
        let error = ErrorResponse::with_details("test_error", "Test message", "Additional info");
        assert_eq!(error.error, "test_error");
        assert_eq!(error.message, "Test message");
        assert_eq!(error.details, Some("Additional info".to_string()));
    }

    #[test]
    fn test_error_response_helpers() {
        let err1 = ErrorResponse::invalid_credentials();
        assert_eq!(err1.error, "invalid_credentials");

        let err2 = ErrorResponse::user_not_found("alice");
        assert_eq!(err2.error, "user_not_found");

        let err3 = ErrorResponse::unauthorized();
        assert_eq!(err3.error, "unauthorized");

        let err4 = ErrorResponse::forbidden("Admin only");
        assert_eq!(err4.error, "forbidden");
    }

    #[test]
    fn test_login_response_serialization() {
        let claims = UserClaims::new("alice", "local", 2000, 1000);
        let response = LoginResponse {
            token: "token123".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 1000,
            claims: UserClaimsResponse::from_claims(claims),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: LoginResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.token, "token123");
        assert_eq!(deserialized.token_type, "Bearer");
        assert_eq!(deserialized.expires_in, 1000);
    }
}

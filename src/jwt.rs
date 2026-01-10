//! JWT token generation and validation.
//!
//! This module handles creating, signing, and validating JWT tokens.
//! Tokens are signed with HS256 and include user claims.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::auth::UserClaims;
use crate::error::AuthError;

/// JWT token with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// The encoded JWT token string
    pub token: String,
    /// When the token expires (Unix timestamp)
    pub expires_at: i64,
    /// Seconds until expiration
    pub ttl: i64,
}

impl Token {
    /// Check if this token is expired.
    pub fn is_expired(&self, now: i64) -> bool {
        now >= self.expires_at
    }

    /// Get seconds until expiration.
    pub fn time_to_expiry(&self, now: i64) -> i64 {
        self.expires_at - now
    }
}

/// JWT validator and token manager.
///
/// Handles encoding and decoding JWT tokens using a shared secret.
/// Uses HS256 algorithm for signing.
///
/// # Example
///
/// ```ignore
/// use poem_auth::jwt::JwtValidator;
/// use poem_auth::UserClaims;
///
/// let validator = JwtValidator::new("my-secret-key")?;
///
/// // Create a token
/// let claims = UserClaims::new("alice", "local", 1704067200, 1703980800);
/// let token = validator.generate_token(&claims)?;
///
/// // Verify a token
/// let verified_claims = validator.verify_token(&token.token)?;
/// assert_eq!(verified_claims.sub, "alice");
/// ```
pub struct JwtValidator {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl std::fmt::Debug for JwtValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtValidator").finish()
    }
}

impl JwtValidator {
    /// Create a new JWT validator with a secret key.
    ///
    /// # Arguments
    ///
    /// * `secret` - The secret key for signing and verifying tokens
    ///
    /// # Errors
    ///
    /// Returns `AuthError::JwtError` if the secret is invalid.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let validator = JwtValidator::new("my-secret-key")?;
    /// ```
    pub fn new(secret: &str) -> Result<Self, AuthError> {
        if secret.is_empty() || secret.len() < 16 {
            return Err(AuthError::jwt(
                "JWT secret must be at least 16 characters long",
            ));
        }

        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());

        Ok(Self {
            encoding_key,
            decoding_key,
        })
    }

    /// Generate a JWT token from user claims.
    ///
    /// # Arguments
    ///
    /// * `claims` - The user claims to encode in the token
    ///
    /// # Returns
    ///
    /// A `Token` struct containing the encoded JWT and expiration info.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::JwtError` if encoding fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let claims = UserClaims::new("alice", "local", exp, iat);
    /// let token = validator.generate_token(&claims)?;
    /// println!("Token: {}", token.token);
    /// ```
    pub fn generate_token(&self, claims: &UserClaims) -> Result<Token, AuthError> {
        let token = encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|e| AuthError::jwt(format!("Failed to encode token: {}", e)))?;

        Ok(Token {
            token,
            expires_at: claims.exp,
            ttl: claims.exp - claims.iat,
        })
    }

    /// Verify and decode a JWT token.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token string to verify
    ///
    /// # Returns
    ///
    /// The decoded `UserClaims` if verification succeeds.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InvalidToken` if the token is invalid.
    /// Returns `AuthError::TokenExpired` if the token has expired.
    /// Returns `AuthError::JwtError` for other JWT errors.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let claims = validator.verify_token(&token_string)?;
    /// println!("User: {}", claims.sub);
    /// ```
    pub fn verify_token(&self, token: &str) -> Result<UserClaims, AuthError> {
        let validation = Validation::default();

        let data = decode::<UserClaims>(token, &self.decoding_key, &validation).map_err(|e| {
            let err_msg = e.to_string();
            if err_msg.contains("ExpiredSignature") {
                AuthError::TokenExpired
            } else if err_msg.contains("InvalidToken") {
                AuthError::InvalidToken
            } else {
                AuthError::jwt(format!("Token verification failed: {}", e))
            }
        })?;

        Ok(data.claims)
    }

    /// Extract token from Authorization header value.
    ///
    /// Expects "Bearer <token>" format.
    ///
    /// # Arguments
    ///
    /// * `auth_header` - The Authorization header value
    ///
    /// # Returns
    ///
    /// The token string if valid format.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InvalidToken` if format is invalid.
    pub fn extract_token(auth_header: &str) -> Result<&str, AuthError> {
        let parts: Vec<&str> = auth_header.splitn(2, ' ').collect();

        if parts.len() != 2 || parts[0] != "Bearer" {
            return Err(AuthError::InvalidToken);
        }

        Ok(parts[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = JwtValidator::new("my-very-long-secret-key").unwrap();
        assert!(validator.generate_token(&UserClaims::new("alice", "local", 1000, 500)).is_ok());
    }

    #[test]
    fn test_validator_creation_short_secret() {
        assert!(JwtValidator::new("short").is_err());
    }

    #[test]
    fn test_validator_creation_empty_secret() {
        assert!(JwtValidator::new("").is_err());
    }

    #[test]
    fn test_generate_and_verify_token() {
        let validator = JwtValidator::new("my-very-long-secret-key").unwrap();
        let now = chrono::Utc::now().timestamp();
        let exp = now + 3600; // 1 hour in future
        let claims = UserClaims::new("alice", "local", exp, now)
            .with_groups(vec!["admins", "users"]);

        // Generate token
        let token = validator.generate_token(&claims).unwrap();
        assert!(!token.token.is_empty());
        assert_eq!(token.expires_at, exp);
        assert_eq!(token.ttl, 3600);

        // Verify token
        let verified = validator.verify_token(&token.token).unwrap();
        assert_eq!(verified.sub, "alice");
        assert_eq!(verified.provider, "local");
        assert_eq!(verified.groups, vec!["admins", "users"]);
    }

    #[test]
    fn test_verify_invalid_token() {
        let validator = JwtValidator::new("my-very-long-secret-key").unwrap();
        assert!(validator.verify_token("invalid.token.here").is_err());
    }

    #[test]
    fn test_verify_expired_token() {
        let validator = JwtValidator::new("my-very-long-secret-key").unwrap();
        let now = chrono::Utc::now().timestamp();
        let claims = UserClaims::new("alice", "local", now - 100, now - 200);

        let token = validator.generate_token(&claims).unwrap();
        assert!(validator.verify_token(&token.token).is_err());
    }

    #[test]
    fn test_extract_token_valid() {
        let auth_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let token = JwtValidator::extract_token(auth_header).unwrap();
        assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    }

    #[test]
    fn test_extract_token_invalid_format() {
        assert!(JwtValidator::extract_token("InvalidHeader token").is_err());
        assert!(JwtValidator::extract_token("Bearer").is_err());
        assert!(JwtValidator::extract_token("").is_err());
    }

    #[test]
    fn test_token_is_expired() {
        let token = Token {
            token: "test".to_string(),
            expires_at: 1000,
            ttl: 500,
        };

        assert!(!token.is_expired(999));
        assert!(token.is_expired(1000));
        assert!(token.is_expired(1001));
    }

    #[test]
    fn test_token_time_to_expiry() {
        let token = Token {
            token: "test".to_string(),
            expires_at: 1000,
            ttl: 500,
        };

        assert_eq!(token.time_to_expiry(900), 100);
        assert_eq!(token.time_to_expiry(1000), 0);
        assert_eq!(token.time_to_expiry(1100), -100);
    }

    #[test]
    fn test_different_secrets_fail_verification() {
        let validator1 = JwtValidator::new("secret-key-number-one-very-long").unwrap();
        let validator2 = JwtValidator::new("secret-key-number-two-very-long").unwrap();

        let claims = UserClaims::new("alice", "local", 2000, 1000);
        let token = validator1.generate_token(&claims).unwrap();

        // Token signed with validator1 should fail verification with validator2
        assert!(validator2.verify_token(&token.token).is_err());
    }

    #[test]
    fn test_token_roundtrip_with_custom_claims() {
        let validator = JwtValidator::new("my-very-long-secret-key").unwrap();
        let now = chrono::Utc::now().timestamp();
        let iat = now;
        let exp = now + 7200; // 2 hours in future

        let mut claims = UserClaims::new("bob", "ldap", exp, iat);
        claims = claims.with_groups(vec!["developers", "devops", "admins"]);

        let token = validator.generate_token(&claims).unwrap();
        let verified = validator.verify_token(&token.token).unwrap();

        assert_eq!(verified.sub, "bob");
        assert_eq!(verified.provider, "ldap");
        assert_eq!(verified.groups.len(), 3);
        assert_eq!(verified.exp, exp);
        assert_eq!(verified.iat, iat);
    }
}

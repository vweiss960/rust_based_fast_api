//! Password hashing and verification utilities.
//!
//! This module provides secure password hashing using Argon2id.
//! All passwords should be hashed before storage.

use argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;

use crate::error::AuthError;

/// Hash a password using Argon2id.
///
/// Uses secure defaults:
/// - Algorithm: Argon2id
/// - Memory: 19456 KB (~19 MB)
/// - Time cost: 2 iterations
/// - Parallelism: 1 thread
///
/// # Arguments
///
/// * `password` - The plaintext password to hash
///
/// # Returns
///
/// A PHC format hash string that includes the algorithm, parameters, salt, and hash.
///
/// # Example
///
/// ```ignore
/// use poem_auth::password::hash_password;
///
/// let hash = hash_password("my_secure_password")?;
/// // hash: "$argon2id$v=19$m=19456,t=2,p=1$..."
/// ```
///
/// # Errors
///
/// Returns `AuthError::PasswordValidationError` if hashing fails.
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    // Validate password length
    if password.is_empty() || password.len() > 128 {
        return Err(AuthError::PasswordValidationError(
            "Password must be between 1 and 128 characters".to_string(),
        ));
    }

    // Generate a random salt
    let salt = SaltString::generate(&mut OsRng);

    // Create Argon2 instance with recommended parameters
    let argon2 = Argon2::default();

    // Hash the password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            AuthError::PasswordValidationError(format!("Failed to hash password: {}", e))
        })?
        .to_string();

    Ok(password_hash)
}

/// Verify a password against a Argon2id hash.
///
/// # Arguments
///
/// * `password` - The plaintext password to verify
/// * `hash` - The PHC format hash to verify against
///
/// # Returns
///
/// * `Ok(())` if password matches
/// * `Err(AuthError::InvalidCredentials)` if password doesn't match
/// * `Err(AuthError::PasswordValidationError)` if verification fails (invalid hash, etc.)
///
/// # Example
///
/// ```ignore
/// use poem_auth::password::{hash_password, verify_password};
///
/// let hash = hash_password("my_secure_password")?;
/// assert!(verify_password("my_secure_password", &hash).is_ok());
/// assert!(verify_password("wrong_password", &hash).is_err());
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<(), AuthError> {
    // Parse the hash
    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        AuthError::PasswordValidationError(format!("Invalid password hash format: {}", e))
    })?;

    // Create Argon2 instance
    let argon2 = Argon2::default();

    // Verify the password
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AuthError::InvalidCredentials)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let hash = hash_password("test_password").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(hash.contains("m=19456"));
        assert!(hash.contains("t=2"));
        assert!(hash.contains("p=1"));
    }

    #[test]
    fn test_verify_password_success() {
        let password = "test_password";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).is_ok());
    }

    #[test]
    fn test_verify_password_failure() {
        let password = "test_password";
        let hash = hash_password(password).unwrap();
        assert!(verify_password("wrong_password", &hash).is_err());
    }

    #[test]
    fn test_hash_empty_password() {
        assert!(hash_password("").is_err());
    }

    #[test]
    fn test_hash_long_password() {
        let long_password = "a".repeat(200);
        assert!(hash_password(&long_password).is_err());
    }

    #[test]
    fn test_verify_invalid_hash() {
        assert!(verify_password("password", "invalid_hash").is_err());
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let hash1 = hash_password("password1").unwrap();
        let hash2 = hash_password("password1").unwrap();
        // Hashes should be different (different salts) but both should verify
        assert_ne!(hash1, hash2);
        assert!(verify_password("password1", &hash1).is_ok());
        assert!(verify_password("password1", &hash2).is_ok());
    }
}

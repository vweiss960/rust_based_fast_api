//! User claims and token structures.
//!
//! `UserClaims` represents the authenticated user's information that gets
//! encoded into JWT tokens and injected into request handlers.

use serde::{Deserialize, Serialize};

/// Claims about an authenticated user.
///
/// This struct represents all information about an authenticated user that should
/// be included in a JWT token and available to request handlers.
///
/// The standard JWT claims (sub, exp, iat, jti) are included as direct fields,
/// and additional custom claims can be stored in the `extra` field.
///
/// # Example
///
/// ```ignore
/// use poem_auth::UserClaims;
///
/// let claims = UserClaims {
///     sub: "alice".to_string(),
///     groups: vec!["admins".to_string(), "users".to_string()],
///     provider: "ldap".to_string(),
///     exp: 1704067200,
///     iat: 1703980800,
///     jti: "550e8400-e29b-41d4-a716-446655440000".to_string(),
///     extra: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserClaims {
    /// Subject: username or user identifier.
    ///
    /// This is the unique identifier for the authenticated user.
    /// Typically a username or email address.
    pub sub: String,

    /// Groups or roles the user belongs to.
    ///
    /// For local auth, these are user-defined groups.
    /// For LDAP/AD, these are LDAP groups the user is a member of.
    /// Can be used for authorization decisions.
    ///
    /// Example: `["Domain Admins", "API Users", "Sales"]`
    pub groups: Vec<String>,

    /// Which authentication provider authenticated this user.
    ///
    /// Examples: "local", "ldap", "oauth2", "saml"
    /// Useful for logging and authorization decisions based on auth method.
    pub provider: String,

    /// Token expiration time (Unix timestamp).
    ///
    /// Seconds since Unix epoch when this token expires.
    pub exp: i64,

    /// Token issued at time (Unix timestamp).
    ///
    /// Seconds since Unix epoch when this token was created.
    pub iat: i64,

    /// Unique JWT ID (jti claim).
    ///
    /// A unique identifier for this specific token instance.
    /// Can be used for token revocation or tracking.
    pub jti: String,

    /// Additional custom claims.
    ///
    /// Use this field to store provider-specific or application-specific claims
    /// that don't fit into the standard fields above.
    ///
    /// Example: `{"department": "Engineering", "clearance_level": 3}`
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl UserClaims {
    /// Create a new UserClaims with minimal required fields.
    ///
    /// # Arguments
    ///
    /// * `username` - The username (becomes `sub`)
    /// * `provider` - The authentication provider name
    /// * `exp` - Token expiration timestamp
    /// * `iat` - Token issued-at timestamp
    ///
    /// # Example
    ///
    /// ```ignore
    /// use poem_auth::UserClaims;
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// let now = SystemTime::now()
    ///     .duration_since(UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_secs() as i64;
    ///
    /// let claims = UserClaims::new("alice", "local", now + 86400, now);
    /// ```
    pub fn new(username: &str, provider: &str, exp: i64, iat: i64) -> Self {
        Self {
            sub: username.to_string(),
            groups: Vec::new(),
            provider: provider.to_string(),
            exp,
            iat,
            jti: uuid::Uuid::new_v4().to_string(),
            extra: None,
        }
    }

    /// Add groups to the claims.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let claims = UserClaims::new("alice", "local", exp, iat)
    ///     .with_groups(vec!["admins", "users"]);
    /// ```
    pub fn with_groups<S: Into<String>>(mut self, groups: Vec<S>) -> Self {
        self.groups = groups.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Add a single group to the claims.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let claims = UserClaims::new("alice", "local", exp, iat)
    ///     .add_group("admins")
    ///     .add_group("users");
    /// ```
    pub fn add_group<S: Into<String>>(mut self, group: S) -> Self {
        self.groups.push(group.into());
        self
    }

    /// Add custom claims.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use serde_json::json;
    ///
    /// let claims = UserClaims::new("alice", "local", exp, iat)
    ///     .with_extra(json!({"department": "Engineering"}));
    /// ```
    pub fn with_extra(mut self, extra: serde_json::Value) -> Self {
        self.extra = Some(extra);
        self
    }

    /// Check if user has a specific group.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if claims.has_group("admins") {
    ///     // Allow admin action
    /// }
    /// ```
    pub fn has_group(&self, group: &str) -> bool {
        self.groups.iter().any(|g| g == group)
    }

    /// Check if user has any of the specified groups.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if claims.has_any_group(&["admins", "moderators"]) {
    ///     // Allow moderation action
    /// }
    /// ```
    pub fn has_any_group(&self, groups: &[&str]) -> bool {
        self.groups.iter().any(|g| groups.contains(&g.as_str()))
    }

    /// Check if user has all of the specified groups.
    pub fn has_all_groups(&self, groups: &[&str]) -> bool {
        groups.iter().all(|g| self.groups.iter().any(|ug| ug == *g))
    }

    /// Check if the token is expired (based on provided current time).
    ///
    /// # Arguments
    ///
    /// * `now` - Current Unix timestamp
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// let now = SystemTime::now()
    ///     .duration_since(UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_secs() as i64;
    ///
    /// if claims.is_expired(now) {
    ///     // Token is expired
    /// }
    /// ```
    pub fn is_expired(&self, now: i64) -> bool {
        now >= self.exp
    }

    /// Get time remaining until expiration (in seconds).
    ///
    /// Returns negative value if already expired.
    pub fn time_to_expiry(&self, now: i64) -> i64 {
        self.exp - now
    }

    /// Get token age (time since issuance in seconds).
    pub fn age(&self, now: i64) -> i64 {
        now - self.iat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_claims_new() {
        let claims = UserClaims::new("alice", "local", 1000, 500);
        assert_eq!(claims.sub, "alice");
        assert_eq!(claims.provider, "local");
        assert_eq!(claims.exp, 1000);
        assert_eq!(claims.iat, 500);
        assert!(!claims.jti.is_empty());
    }

    #[test]
    fn test_user_claims_with_groups() {
        let claims = UserClaims::new("alice", "local", 1000, 500)
            .with_groups(vec!["admins", "users"]);
        assert_eq!(claims.groups.len(), 2);
        assert!(claims.has_group("admins"));
        assert!(claims.has_group("users"));
        assert!(!claims.has_group("guests"));
    }

    #[test]
    fn test_add_group() {
        let claims = UserClaims::new("alice", "local", 1000, 500)
            .add_group("admins")
            .add_group("users");
        assert_eq!(claims.groups.len(), 2);
    }

    #[test]
    fn test_has_any_group() {
        let claims = UserClaims::new("alice", "local", 1000, 500)
            .with_groups(vec!["users"]);
        assert!(claims.has_any_group(&["admins", "users"]));
        assert!(!claims.has_any_group(&["admins", "moderators"]));
    }

    #[test]
    fn test_has_all_groups() {
        let claims = UserClaims::new("alice", "local", 1000, 500)
            .with_groups(vec!["users", "developers"]);
        assert!(claims.has_all_groups(&["users", "developers"]));
        assert!(!claims.has_all_groups(&["users", "developers", "admins"]));
    }

    #[test]
    fn test_is_expired() {
        let claims = UserClaims::new("alice", "local", 1000, 500);
        assert!(!claims.is_expired(999));
        assert!(claims.is_expired(1000));
        assert!(claims.is_expired(1001));
    }

    #[test]
    fn test_time_to_expiry() {
        let claims = UserClaims::new("alice", "local", 1000, 500);
        assert_eq!(claims.time_to_expiry(900), 100);
        assert_eq!(claims.time_to_expiry(1000), 0);
        assert_eq!(claims.time_to_expiry(1100), -100);
    }

    #[test]
    fn test_age() {
        let claims = UserClaims::new("alice", "local", 1000, 500);
        assert_eq!(claims.age(600), 100);
        assert_eq!(claims.age(500), 0);
    }

    #[test]
    fn test_serialization() {
        let claims = UserClaims::new("alice", "local", 1000, 500)
            .with_groups(vec!["admins"]);
        let json = serde_json::to_string(&claims).unwrap();
        let deserialized: UserClaims = serde_json::from_str(&json).unwrap();

        // Check main fields (JTI will differ since it's random on creation)
        assert_eq!(claims.sub, deserialized.sub);
        assert_eq!(claims.groups, deserialized.groups);
        assert_eq!(claims.provider, deserialized.provider);
        assert_eq!(claims.exp, deserialized.exp);
        assert_eq!(claims.iat, deserialized.iat);
    }
}

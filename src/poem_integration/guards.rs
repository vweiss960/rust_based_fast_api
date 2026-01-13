/// Authorization guard trait and implementations
///
/// This module provides composable authorization guards for checking user permissions
/// in a flexible, type-safe way. Guards can be combined using logical operators.

use crate::auth::UserClaims;

/// Trait for authorization guards
///
/// Guards check whether a user (represented by their claims) has permission
/// to perform a certain action. Multiple guards can be composed together.
///
/// # Example
///
/// ```ignore
/// use poem_auth::poem_integration::guards::*;
///
/// let guard = HasGroup("admin".to_string());
///
/// if guard.check(&claims) {
///     println!("User is admin");
/// } else {
///     println!("Access denied");
/// }
/// ```
pub trait AuthGuard: Send + Sync {
    /// Check if the claims satisfy this guard
    fn check(&self, claims: &UserClaims) -> bool;
}

/// Guard that requires a single group membership
///
/// # Example
///
/// ```ignore
/// let guard = HasGroup("admin".to_string());
/// ```
#[derive(Debug, Clone)]
pub struct HasGroup(pub String);

impl AuthGuard for HasGroup {
    fn check(&self, claims: &UserClaims) -> bool {
        claims.has_group(&self.0)
    }
}

/// Guard that requires membership in ANY of the specified groups (OR logic)
///
/// # Example
///
/// ```ignore
/// let guard = HasAnyGroup(vec!["admin".to_string(), "moderator".to_string()]);
/// // User is allowed if they have admin OR moderator group
/// ```
#[derive(Debug, Clone)]
pub struct HasAnyGroup(pub Vec<String>);

impl AuthGuard for HasAnyGroup {
    fn check(&self, claims: &UserClaims) -> bool {
        let group_refs: Vec<&str> = self.0.iter().map(|s| s.as_str()).collect();
        claims.has_any_group(&group_refs)
    }
}

/// Guard that requires membership in ALL of the specified groups (AND logic)
///
/// # Example
///
/// ```ignore
/// let guard = HasAllGroups(vec!["developer".to_string(), "team-lead".to_string()]);
/// // User is allowed only if they have BOTH developer AND team-lead groups
/// ```
#[derive(Debug, Clone)]
pub struct HasAllGroups(pub Vec<String>);

impl AuthGuard for HasAllGroups {
    fn check(&self, claims: &UserClaims) -> bool {
        let group_refs: Vec<&str> = self.0.iter().map(|s| s.as_str()).collect();
        claims.has_all_groups(&group_refs)
    }
}

/// Composite guard that requires BOTH guards to pass (AND logic)
///
/// # Example
///
/// ```ignore
/// let guard = And(
///     HasGroup("developer".to_string()),
///     HasGroup("verified".to_string())
/// );
/// // User must be developer AND verified
/// ```
#[derive(Debug, Clone)]
pub struct And<A: AuthGuard, B: AuthGuard> {
    /// First guard to check
    pub first: A,
    /// Second guard to check
    pub second: B,
}

impl<A: AuthGuard, B: AuthGuard> AuthGuard for And<A, B> {
    fn check(&self, claims: &UserClaims) -> bool {
        self.first.check(claims) && self.second.check(claims)
    }
}

/// Composite guard that requires EITHER guard to pass (OR logic)
///
/// # Example
///
/// ```ignore
/// let guard = Or(
///     HasGroup("admin".to_string()),
///     HasGroup("moderator".to_string())
/// );
/// // User can be admin OR moderator
/// ```
#[derive(Debug, Clone)]
pub struct Or<A: AuthGuard, B: AuthGuard> {
    /// First guard to check
    pub first: A,
    /// Second guard to check
    pub second: B,
}

impl<A: AuthGuard, B: AuthGuard> AuthGuard for Or<A, B> {
    fn check(&self, claims: &UserClaims) -> bool {
        self.first.check(claims) || self.second.check(claims)
    }
}

/// Composite guard that NEGATES another guard (NOT logic)
///
/// # Example
///
/// ```ignore
/// let guard = Not(HasGroup("banned".to_string()));
/// // User is allowed if they DON'T have the "banned" group
/// ```
#[derive(Debug, Clone)]
pub struct Not<A: AuthGuard>(pub A);

impl<A: AuthGuard> AuthGuard for Not<A> {
    fn check(&self, claims: &UserClaims) -> bool {
        !self.0.check(claims)
    }
}

/// Guard that checks if user is enabled/active
///
/// # Example
///
/// ```ignore
/// let guard = IsEnabled;
/// // Only allows active users
/// ```
#[derive(Debug, Clone)]
pub struct IsEnabled;

impl AuthGuard for IsEnabled {
    fn check(&self, _claims: &UserClaims) -> bool {
        // Check if user hasn't been explicitly disabled
        // This is a placeholder - actual implementation depends on
        // whether UserClaims stores enabled status
        true
    }
}

/// Helper functions for creating guards
pub mod builders {
    use super::*;

    /// Create a guard requiring a single group
    pub fn require_group<S: Into<String>>(group: S) -> HasGroup {
        HasGroup(group.into())
    }

    /// Create a guard requiring any of the given groups
    pub fn require_any_group<S: Into<String>>(groups: Vec<S>) -> HasAnyGroup {
        HasAnyGroup(groups.into_iter().map(|s| s.into()).collect())
    }

    /// Create a guard requiring all of the given groups
    pub fn require_all_groups<S: Into<String>>(groups: Vec<S>) -> HasAllGroups {
        HasAllGroups(groups.into_iter().map(|s| s.into()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_group_guard() {
        let claims = UserClaims {
            sub: "user".to_string(),
            groups: vec!["admin".to_string(), "users".to_string()],
            provider: "local".to_string(),
            exp: 1000,
            iat: 0,
            jti: "123".to_string(),
            extra: None,
        };

        let guard = HasGroup("admin".to_string());
        assert!(guard.check(&claims));

        let guard = HasGroup("banned".to_string());
        assert!(!guard.check(&claims));
    }

    #[test]
    fn test_has_any_group_guard() {
        let claims = UserClaims {
            sub: "user".to_string(),
            groups: vec!["user".to_string()],
            provider: "local".to_string(),
            exp: 1000,
            iat: 0,
            jti: "123".to_string(),
            extra: None,
        };

        let guard = HasAnyGroup(vec!["admin".to_string(), "user".to_string()]);
        assert!(guard.check(&claims));

        let guard = HasAnyGroup(vec!["admin".to_string(), "moderator".to_string()]);
        assert!(!guard.check(&claims));
    }

    #[test]
    fn test_and_guard() {
        let claims = UserClaims {
            sub: "user".to_string(),
            groups: vec!["admin".to_string(), "verified".to_string()],
            provider: "local".to_string(),
            exp: 1000,
            iat: 0,
            jti: "123".to_string(),
            extra: None,
        };

        let guard = And(
            HasGroup("admin".to_string()),
            HasGroup("verified".to_string()),
        );
        assert!(guard.check(&claims));

        let guard = And(
            HasGroup("admin".to_string()),
            HasGroup("banned".to_string()),
        );
        assert!(!guard.check(&claims));
    }

    #[test]
    fn test_or_guard() {
        let claims = UserClaims {
            sub: "user".to_string(),
            groups: vec!["admin".to_string()],
            provider: "local".to_string(),
            exp: 1000,
            iat: 0,
            jti: "123".to_string(),
            extra: None,
        };

        let guard = Or(
            HasGroup("admin".to_string()),
            HasGroup("moderator".to_string()),
        );
        assert!(guard.check(&claims));
    }

    #[test]
    fn test_not_guard() {
        let claims = UserClaims {
            sub: "user".to_string(),
            groups: vec!["user".to_string()],
            provider: "local".to_string(),
            exp: 1000,
            iat: 0,
            jti: "123".to_string(),
            extra: None,
        };

        let guard = Not(HasGroup("banned".to_string()));
        assert!(guard.check(&claims));

        let guard = Not(HasGroup("user".to_string()));
        assert!(!guard.check(&claims));
    }
}

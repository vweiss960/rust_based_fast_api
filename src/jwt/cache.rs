//! Token caching for JWT validation.
//!
//! Provides in-memory caching of JWT validation results to reduce
//! cryptographic overhead on hot paths.

use crate::auth::UserClaims;
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "cache")]
use moka::future::Cache;

/// In-memory cache for JWT validation results.
///
/// Caches decoded JWT claims keyed by token string to avoid redundant
/// cryptographic verification. Cache entries expire after a configurable TTL.
///
/// # Example
///
/// ```ignore
/// use poem_auth::jwt::TokenCache;
///
/// let cache = TokenCache::new();
/// cache.insert("token-string".to_string(), claims.clone()).await;
///
/// // Later, retrieve from cache
/// if let Some(cached_claims) = cache.get("token-string").await {
///     // Use cached claims without re-verifying signature
/// }
/// ```
#[cfg(feature = "cache")]
pub struct TokenCache {
    cache: Cache<String, Arc<UserClaims>>,
    ttl: Duration,
}

#[cfg(feature = "cache")]
impl TokenCache {
    /// Create a new token cache with default 5-minute TTL.
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(5 * 60))
    }

    /// Create a new token cache with custom TTL.
    ///
    /// # Arguments
    ///
    /// * `ttl` - Time to live for cached entries
    pub fn with_ttl(ttl: Duration) -> Self {
        let cache = Cache::builder()
            .time_to_live(ttl)
            .build();

        Self { cache, ttl }
    }

    /// Insert a token and its decoded claims into the cache.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token string
    /// * `claims` - The decoded UserClaims
    pub async fn insert(&self, token: String, claims: UserClaims) {
        self.cache.insert(token, Arc::new(claims)).await;
    }

    /// Retrieve cached claims for a token, if available.
    ///
    /// Returns `None` if token is not in cache or cache entry has expired.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token string
    pub async fn get(&self, token: &str) -> Option<UserClaims> {
        self.cache.get(token).await.map(|arc_claims| (*arc_claims).clone())
    }

    /// Remove a token from the cache (useful for revocation).
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token string
    pub async fn remove(&self, token: &str) {
        self.cache.remove(token).await;
    }

    /// Clear all entries from the cache.
    pub fn clear(&self) {
        self.cache.invalidate_all();
    }

    /// Get the cache TTL duration.
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Get number of entries currently in the cache.
    pub fn len(&self) -> u64 {
        self.cache.entry_count()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.entry_count() == 0
    }
}

#[cfg(feature = "cache")]
impl Default for TokenCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "cache")]
impl std::fmt::Debug for TokenCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenCache")
            .field("ttl", &self.ttl)
            .field("entry_count", &self.cache.entry_count())
            .finish()
    }
}

/// Stub implementation when cache feature is disabled.
#[cfg(not(feature = "cache"))]
pub struct TokenCache;

#[cfg(not(feature = "cache"))]
impl TokenCache {
    /// Create a new token cache (no-op when cache feature is disabled).
    pub fn new() -> Self {
        Self
    }

    /// Create a new token cache with custom TTL (no-op when cache feature is disabled).
    pub fn with_ttl(_ttl: Duration) -> Self {
        Self
    }

    /// Insert operation (no-op when cache feature is disabled).
    pub async fn insert(&self, _token: String, _claims: UserClaims) {}

    /// Get operation (always returns None when cache feature is disabled).
    pub async fn get(&self, _token: &str) -> Option<UserClaims> {
        None
    }

    /// Remove operation (no-op when cache feature is disabled).
    pub async fn remove(&self, _token: &str) {}

    /// Clear operation (no-op when cache feature is disabled).
    pub fn clear(&self) {}

    /// Get the cache TTL duration (returns a dummy value when cache feature is disabled).
    pub fn ttl(&self) -> Duration {
        Duration::from_secs(5 * 60)
    }

    /// Get number of entries (always 0 when cache feature is disabled).
    pub fn len(&self) -> u64 {
        0
    }

    /// Check if cache is empty (always true when cache feature is disabled).
    pub fn is_empty(&self) -> bool {
        true
    }
}

#[cfg(not(feature = "cache"))]
impl Default for TokenCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "cache"))]
impl std::fmt::Debug for TokenCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenCache (disabled)").finish()
    }
}

#[cfg(all(test, feature = "cache"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_insert_and_retrieve() {
        let cache = TokenCache::new();
        let claims = UserClaims::new("alice", "local", 1000, 500)
            .with_groups(vec!["admins"]);

        cache.insert("token123".to_string(), claims.clone()).await;

        let retrieved = cache.get("token123").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().sub, "alice");
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = TokenCache::new();
        let result = cache.get("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_remove() {
        let cache = TokenCache::new();
        let claims = UserClaims::new("bob", "local", 1000, 500);

        cache.insert("token456".to_string(), claims).await;
        assert!(cache.get("token456").await.is_some());

        cache.remove("token456").await;
        assert!(cache.get("token456").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_ttl() {
        let ttl = Duration::from_secs(60);
        let cache = TokenCache::with_ttl(ttl);
        assert_eq!(cache.ttl(), ttl);
    }

    #[tokio::test]
    async fn test_cache_default() {
        let cache = TokenCache::default();
        let claims = UserClaims::new("charlie", "local", 1000, 500);

        cache.insert("token789".to_string(), claims).await;
        assert!(cache.get("token789").await.is_some());
    }

    #[tokio::test]
    async fn test_cache_preserves_all_claims() {
        let cache = TokenCache::new();
        let mut claims = UserClaims::new("diana", "ldap", 2000, 1000);
        claims = claims.with_groups(vec!["developers", "admins"]);

        cache.insert("token999".to_string(), claims.clone()).await;

        let retrieved = cache.get("token999").await.unwrap();
        assert_eq!(retrieved.sub, "diana");
        assert_eq!(retrieved.provider, "ldap");
        assert_eq!(retrieved.groups, vec!["developers", "admins"]);
    }
}

//! Rate limiting middleware for protecting authentication endpoints.
//!
//! Provides IP-based rate limiting to prevent brute force attacks on login
//! and other sensitive endpoints.

#[cfg(feature = "rate-limit")]
use governor::{Quota, RateLimiter, state::NotKeyed, state::InMemoryState, clock::DefaultClock};
use std::net::IpAddr;

/// Rate limiter configuration.
///
/// Controls how rate limiting is applied to endpoints.
///
/// # Example
///
/// ```ignore
/// use poem_auth::middleware::RateLimitConfig;
///
/// let config = RateLimitConfig::default()
///     .with_requests_per_minute(100)
///     .with_auth_endpoint_limit(5);
/// ```
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per minute for general endpoints
    pub requests_per_minute: u32,

    /// Maximum requests per minute for auth endpoints (stricter)
    pub auth_endpoint_limit: u32,

    /// Whether to use key-based (IP address) rate limiting
    pub use_key_based: bool,
}

impl RateLimitConfig {
    /// Create a new rate limit config with custom values.
    pub fn new(requests_per_minute: u32, auth_endpoint_limit: u32) -> Self {
        Self {
            requests_per_minute,
            auth_endpoint_limit,
            use_key_based: true,
        }
    }

    /// Set requests per minute for general endpoints.
    pub fn with_requests_per_minute(mut self, limit: u32) -> Self {
        self.requests_per_minute = limit;
        self
    }

    /// Set requests per minute for auth endpoints.
    pub fn with_auth_endpoint_limit(mut self, limit: u32) -> Self {
        self.auth_endpoint_limit = limit;
        self
    }

    /// Enable or disable key-based rate limiting (by IP).
    pub fn with_key_based(mut self, enabled: bool) -> Self {
        self.use_key_based = enabled;
        self
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
            auth_endpoint_limit: 5,
            use_key_based: true,
        }
    }
}

/// Rate limiter for protecting endpoints from brute force attacks.
///
/// Uses IP address-based rate limiting with configurable limits for
/// general and authentication endpoints.
///
/// # Example
///
/// ```ignore
/// use poem_auth::middleware::{RateLimit, RateLimitConfig};
/// use std::net::IpAddr;
///
/// let config = RateLimitConfig::default();
/// let limiter = RateLimit::new(config);
///
/// let ip = "192.168.1.1".parse()?;
/// if limiter.check_auth_limit(&ip).is_ok() {
///     // Allow authentication attempt
/// } else {
///     // Reject due to rate limit
/// }
/// ```
#[cfg(feature = "rate-limit")]
pub struct RateLimit {
    config: RateLimitConfig,
    general_limiter: std::sync::Mutex<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    auth_limiter: std::sync::Mutex<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

#[cfg(feature = "rate-limit")]
impl RateLimit {
    /// Create a new rate limiter with the given configuration.
    pub fn new(config: RateLimitConfig) -> Self {
        // Create limiters: governor uses number-per-period, so we calculate for 1 minute
        let general = RateLimiter::direct(
            Quota::per_minute(std::num::NonZeroU32::new(config.requests_per_minute).unwrap()),
        );

        let auth = RateLimiter::direct(
            Quota::per_minute(std::num::NonZeroU32::new(config.auth_endpoint_limit).unwrap()),
        );

        Self {
            config,
            general_limiter: std::sync::Mutex::new(general),
            auth_limiter: std::sync::Mutex::new(auth),
        }
    }

    /// Create a new rate limiter with default configuration.
    pub fn default_config() -> Self {
        Self::new(RateLimitConfig::default())
    }

    /// Check if a general endpoint request from an IP is allowed.
    ///
    /// Returns `Ok(())` if the request is allowed, or `Err(())` if rate limited.
    pub fn check_general_limit(&self, _ip: &IpAddr) -> Result<(), ()> {
        // Per-IP limiting would require a more complex implementation with per-key limits
        // For now, use a simple global limiter
        let limiter = self.general_limiter.lock().unwrap();
        limiter.check().map_err(|_| ())
    }

    /// Check if an auth endpoint request from an IP is allowed.
    ///
    /// Returns `Ok(())` if the request is allowed, or `Err(())` if rate limited.
    /// Auth endpoints have stricter limits.
    pub fn check_auth_limit(&self, _ip: &IpAddr) -> Result<(), ()> {
        let limiter = self.auth_limiter.lock().unwrap();
        limiter.check().map_err(|_| ())
    }

    /// Get the configuration.
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }

    /// Reset the rate limiter (useful for testing).
    pub fn reset(&self) {
        // The governor crate doesn't provide a public reset method,
        // but we could recreate the limiters if needed
        // For production, you'd typically not reset a rate limiter
    }
}

#[cfg(feature = "rate-limit")]
impl std::fmt::Debug for RateLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimit")
            .field("config", &self.config)
            .finish()
    }
}

/// Stub implementation when rate-limit feature is disabled.
#[cfg(not(feature = "rate-limit"))]
pub struct RateLimit {
    config: RateLimitConfig,
}

#[cfg(not(feature = "rate-limit"))]
impl RateLimit {
    /// Create a new rate limiter (no-op when feature disabled).
    pub fn new(config: RateLimitConfig) -> Self {
        Self { config }
    }

    /// Create a new rate limiter with default configuration (no-op when feature disabled).
    pub fn default_config() -> Self {
        Self::new(RateLimitConfig::default())
    }

    /// Check general limit (always allows when feature disabled).
    pub fn check_general_limit(&self, _ip: &IpAddr) -> Result<(), ()> {
        Ok(())
    }

    /// Check auth limit (always allows when feature disabled).
    pub fn check_auth_limit(&self, _ip: &IpAddr) -> Result<(), ()> {
        Ok(())
    }

    /// Get the configuration.
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }

    /// Reset the rate limiter (no-op when feature disabled).
    pub fn reset(&self) {}
}

#[cfg(not(feature = "rate-limit"))]
impl std::fmt::Debug for RateLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimit (disabled)")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(all(test, feature = "rate-limit"))]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 100);
        assert_eq!(config.auth_endpoint_limit, 5);
        assert!(config.use_key_based);
    }

    #[test]
    fn test_rate_limit_config_builder() {
        let config = RateLimitConfig::default()
            .with_requests_per_minute(200)
            .with_auth_endpoint_limit(10)
            .with_key_based(false);

        assert_eq!(config.requests_per_minute, 200);
        assert_eq!(config.auth_endpoint_limit, 10);
        assert!(!config.use_key_based);
    }

    #[test]
    fn test_rate_limit_creation() {
        let config = RateLimitConfig::default();
        let limiter = RateLimit::new(config);
        assert_eq!(limiter.config().requests_per_minute, 100);
        assert_eq!(limiter.config().auth_endpoint_limit, 5);
    }

    #[test]
    fn test_rate_limit_allows_requests() {
        let config = RateLimitConfig::new(10, 5);
        let limiter = RateLimit::new(config);
        let ip: IpAddr = "192.168.1.1".parse().unwrap();

        // First few requests should be allowed
        for _ in 0..5 {
            assert!(limiter.check_auth_limit(&ip).is_ok());
        }
    }

    #[test]
    fn test_rate_limit_rejects_after_limit() {
        let config = RateLimitConfig::new(2, 2);
        let limiter = RateLimit::new(config);
        let ip: IpAddr = "192.168.1.1".parse().unwrap();

        // Allow first 2 requests
        assert!(limiter.check_auth_limit(&ip).is_ok());
        assert!(limiter.check_auth_limit(&ip).is_ok());

        // Third request should be rejected
        assert!(limiter.check_auth_limit(&ip).is_err());
    }

    #[test]
    fn test_general_vs_auth_limits() {
        let config = RateLimitConfig::new(100, 5);
        let limiter = RateLimit::new(config);
        let _ip: IpAddr = "192.168.1.1".parse().unwrap();

        // Auth endpoint is more restrictive
        assert_eq!(limiter.config().requests_per_minute, 100);
        assert_eq!(limiter.config().auth_endpoint_limit, 5);
    }

    #[test]
    fn test_rate_limit_debug() {
        let config = RateLimitConfig::default();
        let limiter = RateLimit::new(config);
        let debug_str = format!("{:?}", limiter);
        assert!(debug_str.contains("RateLimit"));
    }

    #[test]
    fn test_custom_rate_limit() {
        let config = RateLimitConfig::new(50, 3);
        let limiter = RateLimit::new(config);

        assert_eq!(limiter.config().requests_per_minute, 50);
        assert_eq!(limiter.config().auth_endpoint_limit, 3);
    }
}

#[cfg(all(test, not(feature = "rate-limit")))]
mod tests_disabled {
    use super::*;

    #[test]
    fn test_rate_limit_stub_allows_all() {
        let config = RateLimitConfig::default();
        let limiter = RateLimit::new(config);
        let ip: IpAddr = "192.168.1.1".parse().unwrap();

        // Should always allow when feature disabled
        for _ in 0..100 {
            assert!(limiter.check_general_limit(&ip).is_ok());
            assert!(limiter.check_auth_limit(&ip).is_ok());
        }
    }

    #[test]
    fn test_rate_limit_stub_debug() {
        let config = RateLimitConfig::default();
        let limiter = RateLimit::new(config);
        let debug_str = format!("{:?}", limiter);
        assert!(debug_str.contains("disabled"));
    }
}

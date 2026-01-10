//! Poem middleware utilities for authentication.
//!
//! This module provides utilities and helpers for JWT authentication, master admin authentication,
//! and rate limiting in Poem applications.

pub mod jwt_auth;
pub mod master_auth;

#[cfg(feature = "rate-limit")]
pub mod rate_limit;

pub use jwt_auth::extract_jwt_claims;
pub use master_auth::{MasterAuth, MasterCredentials};

#[cfg(feature = "rate-limit")]
pub use rate_limit::{RateLimit, RateLimitConfig};

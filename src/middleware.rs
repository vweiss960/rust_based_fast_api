//! Poem middleware utilities for authentication.
//!
//! This module provides utilities and helpers for JWT authentication in Poem applications.

pub mod jwt_auth;

pub use jwt_auth::extract_jwt_claims;

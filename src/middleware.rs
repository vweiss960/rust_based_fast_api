//! Poem middleware utilities for authentication.
//!
//! This module provides utilities and helpers for JWT authentication and master admin authentication
//! in Poem applications.

pub mod jwt_auth;
pub mod master_auth;

pub use jwt_auth::extract_jwt_claims;
pub use master_auth::{MasterAuth, MasterCredentials};

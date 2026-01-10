//! Core authentication traits and types.
//!
//! This module defines the fundamental traits and types used throughout poem_auth.

pub mod provider;
pub mod claims;

pub use provider::AuthProvider;
pub use claims::UserClaims;

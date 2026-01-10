//! API handlers and response types for authentication endpoints.
//!
//! Provides request and response types for implementing REST endpoints for user management,
//! login, and configuration operations.

pub mod types;

pub use types::{LoginRequest, LoginResponse, CreateUserRequest, UpdatePasswordRequest};

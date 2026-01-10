//! Built-in authentication providers.
//!
//! This module provides ready-to-use authentication implementations.

pub mod local;

#[cfg(feature = "ldap")]
pub mod ldap;

pub use local::LocalAuthProvider;

#[cfg(feature = "ldap")]
pub use ldap::{LdapAuthProvider, LdapConfig};

/// Poem web framework integration module
///
/// Provides utilities for integrating poem_auth with the Poem web framework,
/// including shared state management, automatic claim extraction, authorization guards,
/// and login response builders for simplified endpoint implementation.

pub mod app_state;
pub mod extractors;
pub mod guards;
pub mod login_helper;

pub use app_state::PoemAppState;
pub use extractors::*;
pub use guards::{AuthGuard, HasGroup, HasAnyGroup, HasAllGroups, And, Or, Not, IsEnabled};
pub use login_helper::LoginResponseBuilder;

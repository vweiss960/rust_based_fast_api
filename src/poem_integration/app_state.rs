/// Shared application state for Poem web server
///
/// This module provides a convenient way to manage and share authentication
/// components across Poem handlers using a global singleton pattern.

use std::sync::OnceLock;
use std::sync::Arc;
use crate::providers::LocalAuthProvider;
use crate::jwt::JwtValidator;

/// Shared application state containing authentication components
///
/// This struct is designed to be initialized once during app startup and then
/// accessed globally from within handler functions.
///
/// # Example
///
/// ```ignore
/// use poem_auth::poem_integration::PoemAppState;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create app state
///     let app_state = PoemAppState::new("users.db", "my-secret-key").await?;
///     app_state.init()?;
///
///     // In handlers, access it via:
///     let state = PoemAppState::get();
///     state.provider.authenticate(username, password).await?;
/// }
/// ```
#[derive(Clone, Debug)]
pub struct PoemAppState {
    /// Authentication provider (handles login verification)
    pub provider: Arc<LocalAuthProvider>,
    /// JWT validator (generates and validates tokens)
    pub jwt: Arc<JwtValidator>,
}

static APP_STATE: OnceLock<PoemAppState> = OnceLock::new();

impl PoemAppState {
    /// Create a new PoemAppState with database and JWT secret
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to SQLite database file
    /// * `jwt_secret` - Secret key for JWT signing (must be at least 16 characters)
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be created or JWT secret is invalid
    ///
    /// # Example
    ///
    /// ```ignore
    /// let state = PoemAppState::new("users.db", "my-super-secret-key").await?;
    /// state.init()?;
    /// ```
    pub async fn new(
        db_path: &str,
        jwt_secret: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let db = crate::db::sqlite::SqliteUserDb::new(db_path).await?;
        let provider = Arc::new(LocalAuthProvider::new(db));
        let jwt = Arc::new(JwtValidator::new(jwt_secret)?);

        Ok(PoemAppState { provider, jwt })
    }

    /// Initialize the global app state (call once during startup)
    ///
    /// This function stores the current PoemAppState in a global OnceLock,
    /// making it accessible from anywhere in the application via `PoemAppState::get()`.
    ///
    /// # Errors
    ///
    /// Returns an error if the global state has already been initialized
    ///
    /// # Example
    ///
    /// ```ignore
    /// let app_state = PoemAppState::new("users.db", "secret").await?;
    /// app_state.init()?;  // Can only be called once
    /// ```
    pub fn init(self) -> Result<(), Self> {
        APP_STATE.set(self)
    }

    /// Get reference to global app state
    ///
    /// # Panics
    ///
    /// Panics if PoemAppState has not been initialized via `init()`
    ///
    /// # Example
    ///
    /// ```ignore
    /// let state = PoemAppState::get();
    /// state.provider.authenticate(username, password).await?;
    /// ```
    pub fn get() -> &'static PoemAppState {
        APP_STATE.get().expect(
            "PoemAppState not initialized. Call PoemAppState::init() during app startup."
        )
    }

    /// Try to get reference to global app state (returns None if not initialized)
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(state) = PoemAppState::try_get() {
    ///     state.provider.authenticate(username, password).await?;
    /// }
    /// ```
    pub fn try_get() -> Option<&'static PoemAppState> {
        APP_STATE.get()
    }

    /// Get a clone of the LocalAuthProvider for passing to handlers
    pub fn provider(&self) -> Arc<LocalAuthProvider> {
        self.provider.clone()
    }

    /// Get a clone of the JwtValidator for passing to handlers
    pub fn jwt(&self) -> Arc<JwtValidator> {
        self.jwt.clone()
    }
}

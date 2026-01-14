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
    /// Server configuration (host, port, optional TLS)
    pub server_config: Option<crate::config::ServerConfig>,
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

        Ok(PoemAppState { provider, jwt, server_config: None })
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

    /// Get server configuration (host, port) with defaults
    pub fn server_config(&self) -> (String, u16) {
        match &self.server_config {
            Some(cfg) => (cfg.host.clone(), cfg.port),
            None => ("0.0.0.0".to_string(), 3000),
        }
    }

    /// Get TLS configuration if present
    pub fn tls_config(&self) -> Option<&crate::config::TlsConfig> {
        self.server_config.as_ref()?.tls.as_ref()
    }

    /// Check if TLS is enabled
    pub fn tls_enabled(&self) -> bool {
        self.tls_config()
            .map(|tls| tls.enabled)
            .unwrap_or(false)
    }

    /// Create a TcpListener bound to configured host:port
    ///
    /// Returns a TcpListener ready to use. If TLS is configured, the config is validated
    /// at startup, but TLS setup must be done via Poem's listener methods or a reverse proxy.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let state = PoemAppState::get();
    /// let listener = state.create_listener()?;
    /// Server::new(app).run(listener).await?;
    /// ```
    /// Get the listener address string
    pub fn listener_addr(&self) -> String {
        let (host, port) = self.server_config();
        format!("{}:{}", host, port)
    }

    /// Validate TLS configuration if enabled
    pub fn validate_listener_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tls) = self.tls_config() {
            if tls.enabled {
                self.validate_tls_files(tls)?;
            }
        }
        Ok(())
    }

    /// Validate that TLS certificate and key files are readable
    fn validate_tls_files(&self, tls: &crate::config::TlsConfig) -> Result<(), Box<dyn std::error::Error>> {
        // Check that certificate file is readable
        let _ = std::fs::read_to_string(&tls.certificate)
            .map_err(|e| format!("Cannot read certificate file '{}': {}", tls.certificate, e))?;

        // Check that key file is readable
        let _ = std::fs::read_to_string(&tls.key)
            .map_err(|e| format!("Cannot read key file '{}': {}", tls.key, e))?;

        // Check CA chain if specified
        if let Some(ca) = &tls.ca_chain {
            let _ = std::fs::read_to_string(ca)
                .map_err(|e| format!("Cannot read CA chain file '{}': {}", ca, e))?;
        }

        Ok(())
    }
}

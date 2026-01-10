# poem_auth

A comprehensive, extensible authentication and authorization framework for the Poem web framework.

## Features

- **Pluggable authentication architecture** - Easily add new auth methods (OAuth2, SAML, etc.)
- **Multiple built-in providers** - Local database and LDAP/Active Directory support
- **JWT token management** - Secure token generation, validation, and caching
- **Admin APIs** - User management and configuration endpoints
- **Audit logging** - Track authentication events for compliance
- **Rate limiting** - Protect against brute force attacks
- **Secure by default** - Argon2 password hashing, keyring secrets management

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
poem_auth = { version = "0.1", features = ["sqlite"] }
```

Basic usage:

```rust
use poem::{listener::TcpListener, App, get};
use poem_auth::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new()
        .at("/protected", get(handler));

    app.run(TcpListener::bind("127.0.0.1:3000")).await?;
    Ok(())
}

async fn handler(claims: UserClaims) -> String {
    format!("Hello, {}!", claims.sub)
}
```

## Documentation

- [AuthProvider Trait](https://docs.rs/poem_auth/latest/poem_auth/trait.AuthProvider.html)
- [UserClaims](https://docs.rs/poem_auth/latest/poem_auth/struct.UserClaims.html)
- [UserDatabase](https://docs.rs/poem_auth/latest/poem_auth/trait.UserDatabase.html)

## Feature Flags

- `cache` (default) - In-memory token caching with moka
- `sqlite` - SQLite user database support
- `ldap` - LDAP/Active Directory support (requires OpenSSL)
- `keyring-support` - OS keyring integration for secrets (requires OpenSSL)
- `rate-limit` - Rate limiting middleware
- `cors` - CORS support

## Security

This crate follows security best practices:

- Passwords hashed with Argon2id
- JWT secrets stored in OS keyrings
- All authentication attempts logged
- Rate limiting against brute force attacks
- Configurable token expiration

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

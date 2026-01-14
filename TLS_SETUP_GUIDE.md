# TLS/HTTPS Configuration Guide

## Overview

The poem_auth crate now supports automatic HTTPS/TLS configuration through the `auth.toml` configuration file. Users can enable encrypted connections with just a few lines of configuration and zero code changes.

## Quick Start: Enable HTTPS in 3 Steps

### Step 1: Generate Test Certificates

For development/testing purposes, generate self-signed certificates:

```bash
cd examples/poem_example

# Create certificates directory
mkdir -p certs

# Generate self-signed certificate (valid for 365 days)
openssl req -x509 -newkey rsa:4096 \
  -keyout certs/server-key.pem \
  -out certs/server.pem \
  -days 365 -nodes \
  -subj "/CN=localhost"
```

### Step 2: Update auth.toml

Add TLS configuration to your `auth.toml`:

```toml
[server]
host = "0.0.0.0"
port = 3000

[server.tls]
enabled = true
certificate = "certs/server.pem"
key = "certs/server-key.pem"
# ca_chain = "certs/ca-chain.pem"  # optional
```

### Step 3: Run Your App

Simply restart your application - TLS is now active:

```bash
cargo run --example poem_example
```

On startup, you'll see:

```
Server: https://0.0.0.0:3000
  🔒 TLS: Enabled
      Certificate: certs/server.pem
      Key: certs/server-key.pem
```

## Configuration Options

### Required Fields (when TLS enabled)

```toml
[server.tls]
enabled = true                     # Must be true to activate TLS
certificate = "path/to/cert.pem"   # X.509 certificate in PEM format
key = "path/to/key.pem"            # Private key in PEM format (PKCS#8)
```

### Optional Fields

```toml
ca_chain = "path/to/ca-chain.pem"  # CA certificate chain (for client verification)
```

## Certificate Formats

### Supported Formats

- **Certificate**: X.509 PEM format (`.pem`, `.crt`)
- **Private Key**: PKCS#8 PEM format (`.pem`, `.key`)
- **CA Chain**: PEM format (optional, for client authentication)

### Converting Existing Certificates

**From PKCS#12 (`.p12` or `.pfx`):**

```bash
# Extract certificate
openssl pkcs12 -in certificate.p12 -out server.pem -clcerts -nokeys

# Extract private key
openssl pkcs12 -in certificate.p12 -out server-key.pem -nocerts -nodes
```

**From DER format:**

```bash
openssl x509 -inform DER -in cert.der -out server.pem
openssl pkcs8 -inform DER -in key.der -out server-key.pem
```

## Production Setup

### Using Let's Encrypt with Certbot

```bash
# Install certbot
sudo apt-get install certbot

# Get certificate for your domain
sudo certbot certonly --standalone -d yourdomain.com

# Update auth.toml to point to Let's Encrypt paths
[server.tls]
enabled = true
certificate = "/etc/letsencrypt/live/yourdomain.com/fullchain.pem"
key = "/etc/letsencrypt/live/yourdomain.com/privkey.pem"
```

### Automatic Certificate Renewal

Let's Encrypt certificates expire after 90 days. Set up automatic renewal:

```bash
# Test renewal (dry run)
sudo certbot renew --dry-run

# Enable auto-renewal timer (most systems)
sudo systemctl enable certbot.timer
```

### Running Behind a Reverse Proxy

If you're using a reverse proxy (nginx, Apache, etc.) for TLS termination:

1. **Option 1**: Keep TLS disabled in poem_auth, configure TLS in reverse proxy
   ```toml
   [server]
   host = "127.0.0.1"
   port = 3000
   # Don't configure TLS section
   ```

2. **Option 2**: Use TLS in both (defense in depth)
   ```toml
   [server.tls]
   enabled = true
   certificate = "/path/to/cert.pem"
   key = "/path/to/key.pem"
   ```

### Security Best Practices

1. **Restrict File Permissions**:
   ```bash
   chmod 600 certs/server-key.pem
   chmod 644 certs/server.pem
   ```

2. **Use Strong Certificates**:
   - Minimum 2048-bit RSA (4096 recommended)
   - Use Let's Encrypt or a trusted CA
   - Avoid self-signed certificates in production

3. **Monitor Expiration**:
   ```bash
   # Check certificate expiration
   openssl x509 -in server.pem -text -noout | grep -A 2 "Validity"
   ```

4. **Rotate Keys Periodically**:
   - Rotate certificates annually
   - Immediately if private key is compromised

## Troubleshooting

### "TLS certificate not found"

**Error**: `Failed to initialize PoemAppState - TLS certificate not found: certs/server.pem`

**Solution**:
- Verify file path is correct and relative to where you run the app
- Use absolute paths if running from different directories
- Check file exists: `ls -la path/to/server.pem`

### "TLS key not found"

**Error**: `Failed to initialize PoemAppState - TLS key not found: certs/server-key.pem`

**Solution**:
- Ensure both certificate and key files exist
- File permissions should allow reading by the app process
- Key must be in PKCS#8 format

### "Cannot read certificate file"

**Error**: `Cannot read certificate file 'certs/server.pem': Permission denied`

**Solution**:
```bash
# Grant read permission
chmod 644 certs/server.pem certs/server-key.pem

# Or run with appropriate privileges
sudo ./my_app
```

### Testing with Self-Signed Certificates

When testing with self-signed certificates, bypass certificate verification:

```bash
# curl - skip verification
curl --insecure https://localhost:3000/

# Python requests
import requests
requests.get('https://localhost:3000/', verify=False)

# wget
wget --no-check-certificate https://localhost:3000/
```

### Certificate Chain Issues

If you get certificate chain errors:

1. Verify the certificate is valid:
   ```bash
   openssl x509 -in server.pem -text -noout
   ```

2. Check that certificate matches the domain:
   ```bash
   openssl x509 -in server.pem -noout -subject
   ```

3. For intermediate certificates, ensure full chain is included:
   ```bash
   cat intermediate.pem >> server.pem
   ```

## API Reference

### Configuration Structures

```rust
pub struct TlsConfig {
    pub enabled: bool,           // Enable/disable TLS
    pub certificate: String,      // Path to certificate file
    pub key: String,              // Path to private key file
    pub ca_chain: Option<String>, // Optional CA chain path
}

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls: Option<TlsConfig>,
}
```

### PoemAppState Methods

```rust
impl PoemAppState {
    /// Get server address (host:port)
    pub fn listener_addr(&self) -> String { ... }

    /// Check if TLS is enabled
    pub fn tls_enabled(&self) -> bool { ... }

    /// Get TLS configuration if present
    pub fn tls_config(&self) -> Option<&TlsConfig> { ... }

    /// Get server configuration (host, port)
    pub fn server_config(&self) -> (String, u16) { ... }

    /// Validate TLS configuration (called at startup)
    pub fn validate_listener_config(&self) -> Result<(), ...> { ... }
}
```

## Examples

### Example 1: Development with Self-Signed Cert

```bash
# Generate cert
openssl req -x509 -newkey rsa:2048 -days 365 -nodes \
  -out certs/server.pem -keyout certs/server-key.pem \
  -subj "/CN=localhost"

# auth.toml
[server.tls]
enabled = true
certificate = "certs/server.pem"
key = "certs/server-key.pem"

# Test
curl --insecure https://localhost:3000/
```

### Example 2: Production with Let's Encrypt

```bash
# auth.toml
[server.tls]
enabled = true
certificate = "/etc/letsencrypt/live/example.com/fullchain.pem"
key = "/etc/letsencrypt/live/example.com/privkey.pem"

# App auto-uses latest certificate
./myapp
```

### Example 3: Using with Docker

```dockerfile
FROM rust:latest

WORKDIR /app
COPY . .

RUN cargo build --release

# Copy certificates
COPY certs/ /app/certs/

# Run with TLS
EXPOSE 3000
CMD ["/app/target/release/poem_example"]
```

```bash
# Build
docker build -t myapp .

# Run
docker run -p 3000:3000 myapp
```

## Limitations & Future Work

### Current Limitations

- TLS config is validated at startup but not hot-reloaded
- Single certificate per server (no SNI support yet)
- No client certificate authentication by default

### Planned Features

- [ ] Hot reloading of certificates
- [ ] SNI (Server Name Indication) for multiple domains
- [ ] Automatic Let's Encrypt renewal integration
- [ ] Client certificate authentication
- [ ] OCSP stapling

## FAQ

**Q: Can I use TLS with a reverse proxy?**
A: Yes! You can either:
1. Use TLS only in the reverse proxy (simpler)
2. Use TLS in both layers (more secure)

**Q: How do I rotate certificates?**
A: Update the paths in `auth.toml` and restart the app. For Let's Encrypt, certbot handles rotation automatically.

**Q: What if the certificate file is not readable?**
A: The app will show a clear error at startup. Check file permissions with `ls -la`.

**Q: Can I use the same certificate for multiple apps?**
A: Yes, just point multiple `auth.toml` files to the same certificate paths.

**Q: How do I test HTTPS locally?**
A: Generate self-signed certificate and use `curl --insecure https://localhost:3000/` or configure your client to trust the cert.

## Resources

- [Let's Encrypt](https://letsencrypt.org/) - Free SSL/TLS certificates
- [Mozilla SSL Configuration Generator](https://ssl-config.mozilla.org/) - SSL/TLS best practices
- [OWASP TLS Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Transport_Layer_Protection_Cheat_Sheet.html)
- [Certbot Documentation](https://certbot.eff.org/)
- [OpenSSL Documentation](https://www.openssl.org/docs/)

## Support

For issues or questions about TLS configuration:

1. Check the troubleshooting section above
2. Verify file paths and permissions
3. Check startup logs for validation errors
4. Ensure certificates are in correct PEM format

# Testing poem_auth Example Application

This document explains how to test the poem_auth example endpoints.

## Quick Start

### Terminal 1: Start the Server

```bash
cd examples/poem_example
cargo run
```

You should see output like:
```
=== poem + poem_auth Example ===

🚀 Server running at http://0.0.0.0:3000

Available endpoints:
  GET  http://localhost:3000/                    - Health check
  GET  http://localhost:3000/hello/:name         - Public greeting
  POST http://localhost:3000/login               - Login to get token
  GET  http://localhost:3000/protected           - Protected endpoint (Phase 2: auto extraction)
  ...
```

### Terminal 2: Run the Test Script

```bash
cd examples/poem_example
python3 test_endpoints.py
```

That's it! The script will test all endpoints automatically.

## Test Script Features

The `test_endpoints.py` script automatically:

1. **Logs in all test users** with their credentials
2. **Tests public endpoints** (no authentication required)
3. **Tests Phase 2 endpoints** (automatic claims extraction + manual guards)
4. **Tests Phase 2B endpoints** (declarative macro-based authorization)
5. **Verifies permission levels** for each user
6. **Reports results** with pass/fail status

### Test Users

Four test users are configured in `auth.toml`:

| Username | Password      | Groups                           | Description              |
|----------|---------------|----------------------------------|--------------------------|
| alice    | password123   | users, admins                    | Admin user               |
| bob      | secret456     | users                            | Regular user             |
| charlie  | mod123456     | users, moderators                | Moderator user           |
| dave     | dev123456     | users, developers, verified      | Developer user           |

## Test Endpoints

### Public Endpoints (No Authentication)

- `GET /` - Health check
- `GET /hello/:name` - Greeting endpoint

### Login

- `POST /login` - Get JWT token

### Phase 2: Protected Endpoints (Manual Guards)

- `GET /protected` - Requires any authenticated user
- `GET /admin` - Requires 'admins' group
- `GET /moderator` - Requires 'admins' OR 'moderators' group

### Phase 2B: Macro-Based Endpoints (Declarative Authorization)

- `GET /admin/macro` - `#[require_group("admins")]`
- `GET /moderator/macro` - `#[require_any_groups("admins", "moderators")]`
- `GET /dev/macro` - `#[require_all_groups("developers", "verified")]`

## Expected Test Results

### For alice (admin user):
- ✅ All endpoints pass (200 OK)
- Has access to: protected, admin, moderator, admin/macro, moderator/macro
- Denied access to: dev/macro (missing 'developers' group)

### For bob (regular user):
- ✅ Public endpoints pass
- ✅ protected endpoint passes (authenticated)
- ❌ admin endpoints fail (403 Forbidden - missing 'admins' group)
- ❌ moderator endpoints fail (403 Forbidden - missing required groups)
- ❌ dev/macro fails (403 Forbidden - missing required groups)

### For charlie (moderator):
- ✅ Public endpoints pass
- ✅ protected endpoint passes
- ❌ admin endpoints fail (missing 'admins' group)
- ✅ moderator endpoints pass (has 'moderators' group)
- ❌ dev/macro fails (missing required groups)

### For dave (developer):
- ✅ Public endpoints pass
- ✅ protected endpoint passes
- ❌ admin endpoints fail (missing 'admins' group)
- ❌ moderator endpoints fail (missing required groups)
- ✅ dev/macro passes (has both 'developers' and 'verified')

## Manual Testing with curl

If you prefer to test manually with curl:

### Login and get token

```bash
# Login as alice (admin)
TOKEN=$(curl -X POST http://localhost:3000/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"password123"}' | jq -r '.token')

echo "Token: $TOKEN"
```

### Test protected endpoint

```bash
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/protected

# Expected response:
# {
#   "message": "Access granted",
#   "username": "alice",
#   "groups": ["users", "admins"],
#   "expires_in": 86400
# }
```

### Test admin endpoint (manual guard)

```bash
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/admin

# Expected response (alice has 'admins' group):
# {
#   "message": "Admin access granted",
#   "username": "alice",
#   "admin_group": "admins"
# }
```

### Test admin endpoint with non-admin token

```bash
# Login as bob (regular user)
BOB_TOKEN=$(curl -X POST http://localhost:3000/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"bob","password":"secret456"}' | jq -r '.token')

curl -H "Authorization: Bearer $BOB_TOKEN" http://localhost:3000/admin

# Expected response (bob doesn't have 'admins' group):
# {
#   "error": "This endpoint requires 'admins' group membership"
# }
```

### Test macro-protected endpoint

```bash
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/admin/macro

# Expected response:
# {
#   "message": "Admin access granted via macro!",
#   "username": "alice"
# }
```

### Test without authentication

```bash
curl http://localhost:3000/protected

# Expected response (401 Unauthorized):
# {
#   "error": "Unauthorized"
# }
```

## Advanced Testing Options

### Test with custom host/port

```bash
python3 test_endpoints.py --host 127.0.0.1 --port 8080
```

### Test specific endpoints only

Edit `test_endpoints.py` and comment out test sections you don't need, or create a custom script that imports the `EndpointTester` class.

## Understanding Test Output

The test script produces colored output:

- ✅ **Green PASS** - Endpoint returned expected status code
- ❌ **Red FAIL** - Endpoint returned unexpected status code
- ⏭️ **Yellow SKIP** - Test was skipped

Example output:
```
>>> PHASE 2: PROTECTED ENDPOINTS (Auto-extraction + Manual Guards)

Endpoint: /protected (requires any authenticated user)
Phase 2 feature: Automatic UserClaims extraction via FromRequest

✅ /protected (alice)                          PASS  - Expected 200, got 200
✅ /protected (bob)                            PASS  - Expected 200, got 200
✅ /protected (charlie)                        PASS  - Expected 200, got 200
✅ /protected (dave)                           PASS  - Expected 200, got 200
✅ /protected [NO AUTH]                        FAIL  - Expected 401, got 401
```

## Troubleshooting

### "Cannot connect to server"

Make sure the server is running:
```bash
cd examples/poem_example
cargo run
```

Wait for the "Server running" message before running the test script.

### "Module 'requests' not found"

Install the requests library:
```bash
pip install requests
```

Or if using Python 3:
```bash
pip3 install requests
```

### Tests are failing unexpectedly

1. Check that the server is running in debug or release mode
2. Check that `auth.toml` is in the `examples/poem_example` directory
3. Check that the database hasn't gotten corrupted (delete `poem_example.db` and restart)

### Want to see more details?

The test script runs the following test suites in order:
1. Public endpoints (health check, hello)
2. Login tests (all users + invalid credentials)
3. Phase 2 protected endpoints (manual guards)
4. Phase 2B macro endpoints (declarative authorization)

## What Each Phase Tests

### Phase 2: Protected Endpoints

Demonstrates:
- Automatic UserClaims extraction via Poem's FromRequest trait
- Manual guard checks inside handler body
- Composable guards (HasAnyGroup for OR logic)
- Proper 403 Forbidden responses

### Phase 2B: Macro-Based Endpoints

Demonstrates:
- `#[require_group(...)]` - Single group check (declarative)
- `#[require_any_groups(...)]` - OR logic (declarative)
- `#[require_all_groups(...)]` - AND logic (declarative)
- Automatic error responses without boilerplate

## Performance Notes

- All tests complete in < 100ms (including network round-trips)
- JWT validation is cached for performance
- Tests can be run repeatedly without issues
- Database is created/initialized automatically

## Integration with CI/CD

The test script can be integrated into CI/CD pipelines:

```bash
#!/bin/bash
set -e

# Build and run server in background
cd examples/poem_example
cargo build --release
timeout 60 ./target/release/poem_example &
SERVER_PID=$!

# Give server time to start
sleep 2

# Run tests
python3 test_endpoints.py

# Cleanup
kill $SERVER_PID || true
```

## Advanced: Extending the Test Script

The `EndpointTester` class is designed to be extended. You can:

1. Add new test methods
2. Customize test users
3. Add new endpoints to test
4. Modify expected status codes

Example:
```python
tester = EndpointTester()
tester.users["custom_user"] = TestUser(
    username="test",
    password="pass",
    groups=["custom_group"],
    description="Custom test user"
)
```

## See Also

- [examples/poem_example/src/main.rs](src/main.rs) - Example application source code
- [examples/poem_example/auth.toml](auth.toml) - Configuration and test users
- [.claude/PHASE2_QUICK_REFERENCE.md](../../.claude/PHASE2_QUICK_REFERENCE.md) - Phase 2 features
- [.claude/PHASE2B_QUICK_REFERENCE.md](../../.claude/PHASE2B_QUICK_REFERENCE.md) - Phase 2B features

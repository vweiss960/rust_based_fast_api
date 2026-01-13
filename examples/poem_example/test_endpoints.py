#!/usr/bin/env python3
"""
Comprehensive endpoint tester for poem_auth example application.

Tests all endpoints across different users with varying permission levels.
Demonstrates Phase 1, Phase 2, and Phase 2B features in action.

Usage:
    python3 test_endpoints.py

    # With custom host/port:
    python3 test_endpoints.py --host localhost --port 3000
"""

import requests
import sys
import argparse
import json
from dataclasses import dataclass
from typing import Dict, Optional, Tuple
from enum import Enum


class Color:
    """ANSI color codes for terminal output."""
    RESET = '\033[0m'
    BOLD = '\033[1m'
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'


class TestResult(Enum):
    """Test result status."""
    PASS = "PASS"
    FAIL = "FAIL"
    SKIP = "SKIP"


@dataclass
class TestUser:
    """Represents a test user with credentials and expected permissions."""
    username: str
    password: str
    groups: list
    description: str

    def has_group(self, group: str) -> bool:
        """Check if user has a specific group."""
        return group in self.groups


class EndpointTester:
    """Tests all poem_auth example endpoints."""

    def __init__(self, host: str = "localhost", port: int = 3000):
        self.base_url = f"http://{host}:{port}"
        self.session = requests.Session()
        self.test_results = []
        self.tokens = {}

        # Define test users matching auth.toml
        self.users = {
            "alice": TestUser(
                username="alice",
                password="password123",
                groups=["users", "admins"],
                description="Admin user (all permissions)"
            ),
            "bob": TestUser(
                username="bob",
                password="secret456",
                groups=["users"],
                description="Regular user (minimal permissions)"
            ),
            "charlie": TestUser(
                username="charlie",
                password="mod123456",
                groups=["users", "moderators"],
                description="Moderator (mod permissions)"
            ),
            "dave": TestUser(
                username="dave",
                password="dev123456",
                groups=["users", "developers", "verified"],
                description="Developer (dev permissions)"
            ),
        }

    def print_header(self, text: str):
        """Print a colored header."""
        print(f"\n{Color.BOLD}{Color.CYAN}{'=' * 80}{Color.RESET}")
        print(f"{Color.BOLD}{Color.CYAN}{text.center(80)}{Color.RESET}")
        print(f"{Color.BOLD}{Color.CYAN}{'=' * 80}{Color.RESET}\n")

    def print_section(self, text: str):
        """Print a colored section header."""
        print(f"\n{Color.BOLD}{Color.BLUE}>>> {text}{Color.RESET}")

    def print_test(self, test_name: str, result: TestResult, message: str = ""):
        """Print a test result."""
        status_color = Color.GREEN if result == TestResult.PASS else Color.RED if result == TestResult.FAIL else Color.YELLOW
        status_icon = "✅" if result == TestResult.PASS else "❌" if result == TestResult.FAIL else "⏭️"

        print(f"{status_icon} {test_name:<50} {status_color}{result.value:>8}{Color.RESET}", end="")
        if message:
            print(f" - {message}", end="")
        print()

        self.test_results.append((test_name, result))

    def login(self, user: TestUser) -> Optional[str]:
        """Login a user and return JWT token."""
        endpoint = f"{self.base_url}/login"
        payload = {
            "username": user.username,
            "password": user.password
        }

        try:
            response = self.session.post(endpoint, json=payload, timeout=5)

            if response.status_code == 200:
                data = response.json()
                token = data.get("token")
                if token:
                    self.tokens[user.username] = token
                    return token

            return None
        except Exception as e:
            print(f"Login error: {e}")
            return None

    def test_endpoint(self, endpoint: str, method: str = "GET",
                     user: Optional[TestUser] = None,
                     expected_status: int = 200,
                     description: str = "") -> bool:
        """Test an endpoint and return True if it matches expected status."""
        url = f"{self.base_url}{endpoint}"

        try:
            headers = {}
            if user and user.username in self.tokens:
                token = self.tokens[user.username]
                headers["Authorization"] = f"Bearer {token}"

            if method == "GET":
                response = self.session.get(url, headers=headers, timeout=5)
            elif method == "POST":
                response = self.session.post(url, headers=headers, timeout=5)
            else:
                return False

            success = response.status_code == expected_status

            # Format test name
            user_str = f" ({user.username})" if user else ""
            auth_str = " [NO AUTH]" if not user or user.username not in self.tokens else ""
            test_name = f"{endpoint}{user_str}{auth_str}"

            result = TestResult.PASS if success else TestResult.FAIL
            message = f"Expected {expected_status}, got {response.status_code}"

            self.print_test(test_name, result, message)

            return success
        except Exception as e:
            user_str = f" ({user.username})" if user else ""
            self.print_test(f"{endpoint}{user_str}", TestResult.FAIL, f"Error: {str(e)}")
            return False

    def run_public_endpoint_tests(self):
        """Test public endpoints that don't require authentication."""
        self.print_section("PUBLIC ENDPOINTS (No authentication required)")

        # Health check
        self.test_endpoint("/", "GET", None, 200, "Health check")

        # Hello endpoint
        self.test_endpoint("/hello/World", "GET", None, 200, "Greeting endpoint")

        # Invalid routes should return 404
        self.test_endpoint("/nonexistent", "GET", None, 404, "Non-existent endpoint")

    def run_login_tests(self):
        """Test login endpoint with all users."""
        self.print_section("LOGIN ENDPOINT Tests")

        print("\nLogging in test users...\n")

        for username, user in self.users.items():
            endpoint = "/login"
            url = f"{self.base_url}{endpoint}"
            payload = {
                "username": user.username,
                "password": user.password
            }

            try:
                response = self.session.post(url, json=payload, timeout=5)
                if response.status_code == 200:
                    data = response.json()
                    self.tokens[username] = data.get("token")
                    self.print_test(
                        f"Login {username}: {user.description}",
                        TestResult.PASS,
                        f"Token received (groups: {', '.join(user.groups)})"
                    )
                else:
                    self.print_test(f"Login {username}", TestResult.FAIL, f"Status {response.status_code}")
            except Exception as e:
                self.print_test(f"Login {username}", TestResult.FAIL, str(e))

        # Test invalid credentials
        print()
        invalid_payload = {"username": "alice", "password": "wrongpassword"}
        try:
            response = self.session.post(f"{self.base_url}/login", json=invalid_payload, timeout=5)
            result = TestResult.PASS if response.status_code == 401 else TestResult.FAIL
            self.print_test("Login with invalid password", result, f"Status {response.status_code}")
        except Exception as e:
            self.print_test("Login with invalid password", TestResult.FAIL, str(e))

    def run_phase2_protected_endpoint_tests(self):
        """Test Phase 2 endpoints (auto-extraction + manual guards)."""
        self.print_section("PHASE 2: PROTECTED ENDPOINTS (Auto-extraction + Manual Guards)")

        # /protected - requires any authenticated user
        print(f"\n{Color.BOLD}Endpoint: /protected{Color.RESET} (requires any authenticated user)")
        print("Phase 2 feature: Automatic UserClaims extraction via FromRequest\n")

        for user in self.users.values():
            self.test_endpoint("/protected", "GET", user, 200)

        # Test without token
        self.test_endpoint("/protected", "GET", None, 401, "No authentication")

        # /admin - requires 'admins' group (manual guard check)
        print(f"\n{Color.BOLD}Endpoint: /admin{Color.RESET} (requires 'admins' group)")
        print("Phase 2 feature: Manual guard check inside handler\n")

        for user in self.users.values():
            expected = 200 if "admins" in user.groups else 403
            self.test_endpoint("/admin", "GET", user, expected)

        # /moderator - requires 'admins' OR 'moderators' group
        print(f"\n{Color.BOLD}Endpoint: /moderator{Color.RESET} (requires 'admins' OR 'moderators' group)")
        print("Phase 2 feature: Composable guards (HasAnyGroup)\n")

        for user in self.users.values():
            expected = 200 if ("admins" in user.groups or "moderators" in user.groups) else 403
            self.test_endpoint("/moderator", "GET", user, expected)

    def run_phase2b_macro_endpoint_tests(self):
        """Test Phase 2B endpoints (procedural macro-based guards)."""
        self.print_section("PHASE 2B: MACRO-BASED ENDPOINTS (Declarative Authorization)")

        # /admin/macro - #[require_group("admins")]
        print(f"\n{Color.BOLD}Endpoint: /admin/macro{Color.RESET} (macro: #[require_group(\"admins\")])")
        print("Phase 2B feature: Declarative single-group authorization\n")

        for user in self.users.values():
            expected = 200 if "admins" in user.groups else 403
            self.test_endpoint("/admin/macro", "GET", user, expected)

        # /moderator/macro - #[require_any_groups("admins", "moderators")]
        print(f"\n{Color.BOLD}Endpoint: /moderator/macro{Color.RESET} (macro: #[require_any_groups(\"admins\", \"moderators\")])")
        print("Phase 2B feature: Declarative OR-logic authorization\n")

        for user in self.users.values():
            expected = 200 if ("admins" in user.groups or "moderators" in user.groups) else 403
            self.test_endpoint("/moderator/macro", "GET", user, expected)

        # /dev/macro - #[require_all_groups("developers", "verified")]
        print(f"\n{Color.BOLD}Endpoint: /dev/macro{Color.RESET} (macro: #[require_all_groups(\"developers\", \"verified\")])")
        print("Phase 2B feature: Declarative AND-logic authorization\n")

        for user in self.users.values():
            expected = 200 if ("developers" in user.groups and "verified" in user.groups) else 403
            self.test_endpoint("/dev/macro", "GET", user, expected)

    def run_all_tests(self):
        """Run all test suites."""
        self.print_header("POEM_AUTH EXAMPLE - COMPREHENSIVE ENDPOINT TEST SUITE")

        print(f"{Color.BOLD}Testing endpoint: {self.base_url}{Color.RESET}")
        print(f"\nTest Users:")
        for user in self.users.values():
            print(f"  • {user.username:10} - {user.description:40} Groups: {', '.join(user.groups)}")

        try:
            # Test public endpoints first
            self.run_public_endpoint_tests()

            # Login all users
            self.run_login_tests()

            # Test Phase 2 endpoints
            self.run_phase2_protected_endpoint_tests()

            # Test Phase 2B endpoints
            self.run_phase2b_macro_endpoint_tests()

            # Print summary
            self.print_summary()

        except KeyboardInterrupt:
            print(f"\n\n{Color.YELLOW}Tests interrupted by user{Color.RESET}")
            self.print_summary()
            sys.exit(1)

    def print_summary(self):
        """Print test summary."""
        self.print_header("TEST SUMMARY")

        total = len(self.test_results)
        passed = sum(1 for _, result in self.test_results if result == TestResult.PASS)
        failed = sum(1 for _, result in self.test_results if result == TestResult.FAIL)
        skipped = sum(1 for _, result in self.test_results if result == TestResult.SKIP)

        print(f"Total Tests:  {Color.BOLD}{total}{Color.RESET}")
        print(f"Passed:       {Color.BOLD}{Color.GREEN}{passed}{Color.RESET}")
        print(f"Failed:       {Color.BOLD}{Color.RED}{failed}{Color.RESET}")
        print(f"Skipped:      {Color.BOLD}{Color.YELLOW}{skipped}{Color.RESET}")

        if failed > 0:
            print(f"\n{Color.BOLD}{Color.RED}Failed Tests:{Color.RESET}")
            for name, result in self.test_results:
                if result == TestResult.FAIL:
                    print(f"  ❌ {name}")
            return False
        else:
            print(f"\n{Color.BOLD}{Color.GREEN}All tests passed! ✅{Color.RESET}")
            return True

    def check_server_availability(self) -> bool:
        """Check if the server is running."""
        try:
            response = self.session.get(f"{self.base_url}/", timeout=2)
            return True
        except requests.exceptions.ConnectionError:
            print(f"{Color.BOLD}{Color.RED}Error: Cannot connect to server at {self.base_url}{Color.RESET}")
            print(f"Make sure the poem_auth example is running:\n")
            print(f"  cd examples/poem_example")
            print(f"  cargo run\n")
            return False
        except Exception as e:
            print(f"{Color.BOLD}{Color.RED}Error: {str(e)}{Color.RESET}")
            return False


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Test poem_auth example endpoints",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python3 test_endpoints.py
  python3 test_endpoints.py --host localhost --port 3000
  python3 test_endpoints.py --host 127.0.0.1 --port 8080
        """
    )

    parser.add_argument("--host", default="localhost", help="Server host (default: localhost)")
    parser.add_argument("--port", type=int, default=3000, help="Server port (default: 3000)")

    args = parser.parse_args()

    tester = EndpointTester(host=args.host, port=args.port)

    # Check if server is available
    if not tester.check_server_availability():
        sys.exit(1)

    # Run all tests
    tester.run_all_tests()


if __name__ == "__main__":
    main()

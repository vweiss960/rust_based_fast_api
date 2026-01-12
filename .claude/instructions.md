# Project Instructions

## Project Overview
This is a Rust crate that extends the [poem](https://github.com/poem-web/poem) web framework with additional authentication capabilities.

## Purpose
Provide easy-to-use authentication modules for poem-based API applications, including:
- Active Directory authentication
- Other enterprise authentication methods

## Development Guidelines

### Code Quality
- **All changes must include tests** - every modification or new feature requires explicit test coverage
- **Feature-driven testing** - all documented features must have corresponding integration tests
- Follow Rust best practices and idioms
- Ensure code is well-documented with inline rustdoc comments where needed

### Documentation Requirements

- **Documentation updates are managed by the doc-guardian subagent**
- Only create new documentation when explicitly requested
- Use existing files instead of creating new ones
- Ask before creating anything new
- Ask before making major changes to existing documents that don't have to do with feature changes
- Keep things concise and focused
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
  **Ask the user before creating any new documents!!**
- **Before committing**: The doc-guardian subagent reviews all changes and updates documentation only when necessary
- **Consult doc-guardian**: When you have questions about how the codebase works or need to understand existing documentation
- The doc-guardian prevents documentation bloat by only updating when changes affect public APIs or user-facing behavior

### Testing Requirements
- **Testing is managed by the feature-tester subagent**
- **Two-layer approach**: Unit tests for functions, integration tests for features
- **Documentation alignment**: Tests must validate all documented feature behavior
- **Feature-driven**: Every documented capability must have a corresponding feature test
- The feature-tester collaborates with doc-guardian to ensure test-documentation alignment

### Project Structure
- This is designed as an **importable crate** - consider the developer experience when designing APIs
- Maintain clear, intuitive interfaces for users integrating this crate

## Subagent Usage

### Doc-Guardian Subagent (Haiku)
The `doc-guardian` subagent is the expert on all documentation and codebase knowledge.

**When doc-guardian is used automatically:**
- Before any commit to review changes and update documentation if needed
- When you ask questions about how the codebase works
- When you request documentation updates or reviews

**Characteristics:**
- Only updates documentation when changes make current docs outdated
- Prevents documentation sprawl and bloat
- Expert in Rust documentation best practices and authentication domain
- Maintains high documentation quality standards
- Collaborates with feature-tester to ensure documented features are testable

### Feature-Tester Subagent (Sonnet)
The `feature-tester` subagent is the expert on comprehensive testing and test-documentation alignment.

**When feature-tester is used automatically:**
- When new features are implemented
- When existing features are modified
- When documentation is updated to reflect new capabilities
- When comprehensive test coverage is needed

**Characteristics:**
- Creates both unit tests and feature-level integration tests
- Ensures all documented features have corresponding tests
- Collaborates with doc-guardian to validate test-documentation alignment
- Tests from the user's perspective (how the crate will actually be used)
- Flags discrepancies between documented and actual behavior

**Collaboration between subagents:**
```
feature-tester ←→ doc-guardian
     ↓                    ↓
  Tests validate    Documentation defines
  documented behavior    expected features
```

## Development Workflow

### Adding a New Feature

1. **Implement the feature** in code
2. **feature-tester creates tests**:
   - Unit tests for major functions
   - Integration test for complete feature workflow
3. **doc-guardian reviews for documentation**:
   - Updates docs if feature affects public API
   - Ensures documented behavior matches implementation
4. **feature-tester validates alignment**:
   - Confirms tests validate documented behavior
   - Flags any discrepancies
5. **Commit** with comprehensive tests and accurate documentation

### Example Workflow
```bash
# Developer implements OAuth2 refresh tokens
> Implement OAuth2 refresh token support with automatic rotation

# feature-tester is automatically invoked
[Creates integration tests in tests/oauth2_features.rs]
[Creates unit tests for token refresh logic]

# doc-guardian is automatically invoked
[Reviews if README needs OAuth2 refresh token documentation]
[Updates documentation if necessary]

# feature-tester validates alignment
[Confirms tests validate documented refresh token behavior]

# Ready to commit
git add .
git commit -m "Add OAuth2 refresh token support with tests and docs"
```

## Commit Workflow

1. Make your code changes
2. feature-tester creates/updates tests automatically
3. Stage changes with `git add`
4. doc-guardian reviews changes and updates docs if needed
5. feature-tester validates test-documentation alignment
6. Commit with descriptive message
7. Push to repository

## Testing Standards

### Unit Tests (in module files)
- Test major public functions
- Located in `#[cfg(test)]` modules
- Focus on edge cases and error conditions

### Feature Tests (in tests/ directory)
- Test complete authentication flows
- Validate integration with Poem
- Match documented user-facing features
- Serve as executable examples

### Quality Gate
A feature is not complete until:
- ✅ Code is implemented
- ✅ Unit tests for major functions exist
- ✅ Integration test for feature workflow exists
- ✅ Tests validate documented behavior
- ✅ Documentation is accurate and up-to-date
- ✅ feature-tester and doc-guardian confirm alignment

## Questions and Documentation

- For questions about how the codebase works, consult the doc-guardian subagent
- For testing new features, the feature-tester will handle comprehensive test creation
- For documentation updates, the doc-guardian will handle it during pre-commit review
- Trust both subagents to collaborate and maintain quality
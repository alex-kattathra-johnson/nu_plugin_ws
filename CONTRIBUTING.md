# Contributing to nu_plugin_ws

Thank you for contributing to the WebSocket plugin for Nushell!

## Development Setup

### Prerequisites

- Rust toolchain (stable)
- Nushell (for testing the plugin)
- Python (for pre-commit hooks)

### Setting Up Pre-commit Hooks

This project uses [pre-commit](https://pre-commit.com/) to ensure code quality. To set it up:

1. Install pre-commit:
   ```bash
   # Using pip
   pip install pre-commit

   # On macOS using Homebrew
   brew install pre-commit

   # On Ubuntu/Debian
   apt install pre-commit
   ```

2. Install the git hooks:
   ```bash
   pre-commit install
   pre-commit install --hook-type commit-msg  # For commit message linting
   ```

3. (Optional) Run against all files:
   ```bash
   pre-commit run --all-files
   ```

### What the Hooks Do

Before each commit, the following checks will run automatically:

- **cargo fmt**: Ensures consistent code formatting
- **cargo clippy**: Catches common mistakes and suggests improvements
- **cargo check**: Verifies the code compiles
- **cargo test**: Runs all tests
- **File checks**:
  - Removes trailing whitespace
  - Ensures files end with a newline
  - Validates YAML/TOML files
  - Checks for large files (>1MB)
  - Detects merge conflicts
  - Prevents committing private keys

### Bypassing Hooks

If you need to bypass the hooks temporarily (not recommended):
```bash
git commit --no-verify
```

## Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Write tests for new functionality
- Document public APIs

## Testing

Run all tests with:
```bash
cargo test
```

Run specific test suites:
```bash
cargo test --test unit_tests
cargo test --test integration_tests
```

## Commit Messages

We use conventional commits. Examples:
- `feat: add support for custom headers`
- `fix: handle connection timeout properly`
- `docs: update README with examples`
- `test: add edge case tests for malformed URLs`
- `chore: update dependencies`

## Pull Request Process

1. Ensure all pre-commit hooks pass
2. Update documentation if needed
3. Add tests for new features
4. Update CHANGELOG.md if applicable
5. Ensure CI passes on your PR

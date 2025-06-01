# List available commands
default:
    @just --list

# Format code using rustfmt
format:
    @echo "Formatting all code..."
    cargo fmt --all
    find src -name "*.rs" -exec leptosfmt -q {} \;
    @echo "Done formatting!"

# Run clippy to lint the code
lint:
    @echo "Linting with clippy..."
    cargo fmt -- --check
    find . -name "*.rs" -exec leptosfmt -q --check {} \;
    cargo clippy

# Fix linting issues where possible
lint-fix:
    @echo "Fixing linting issues..."
    cargo clippy --fix -- -D warnings

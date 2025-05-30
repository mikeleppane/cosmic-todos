# List available commands
default:
    @just --list

# Format code using rustfmt
format:
    @echo "Formatting all code..."
    cargo fmt --all
    leptosfmt -q src/**/*.rs
    @echo "Done formatting!"

# Run clippy to lint the code
lint:
    @echo "Linting with clippy..."
    leptosfmt -q --check ./**/*.rs
    cargo fmt -- --check
    cargo clippy

# Fix linting issues where possible
lint-fix:
    @echo "Fixing linting issues..."
    cargo clippy --fix -- -D warnings

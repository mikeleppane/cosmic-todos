# List available commands
default:
    @just --list

# Format code using rustfmt
format:
    @echo "Formatting Rust code with rustfmt..."
    cargo fmt
    leptosfmt -q ./**/*.rs 

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

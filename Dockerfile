# If you're using stable, use this instead
FROM rust:1.87 AS builder

# Install cargo-binstall with proper verification
RUN wget -q https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz \
    && tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz \
    && cp cargo-binstall /usr/local/cargo/bin \
    && rm cargo-binstall-x86_64-unknown-linux-musl.tgz cargo-binstall

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends clang curl mold

# Install Node.js
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
RUN wget -qO- https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install --no-install-recommends -y nodejs \
    && node -v \
    && cargo binstall cargo-leptos -y \
    && rustup target add wasm32-unknown-unknown \
    && groupadd --gid 1001 appgroup \
    && useradd --uid 1001 --gid appgroup --shell /bin/bash --create-home appuser \
    && mkdir -p /app && chown appuser:appgroup /app

WORKDIR /app

# Copy dependency files first for better caching
COPY --chown=appuser:appgroup Cargo.toml Cargo.lock ./

# Create dummy source files to satisfy Cargo
RUN mkdir -p src \
    && echo "fn main() {}" > src/main.rs \
    && if grep -q "\\[lib\\]" Cargo.toml; then echo "// dummy lib" > src/lib.rs; fi

# Switch to non-root user for build
USER appuser

# Pre-build dependencies (this layer will be cached)
RUN cargo fetch

# Copy source code
COPY --chown=appuser:appgroup . .

# Build the app
RUN cargo leptos build --release

# Use distroless for minimal runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Create nonroot user in runtime stage
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && groupadd --gid 1000 nonroot \
  && useradd --uid 1000 --gid nonroot --no-create-home --shell /usr/sbin/nologin nonroot \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

# Copy only the necessary files
COPY --from=builder --chown=nonroot:nonroot /app/target/release/cosmic-rust /app/cosmic-rust
COPY --from=builder --chown=nonroot:nonroot /app/site /app/site

# Copy Cargo.toml if it's needed at runtime
COPY --from=builder --chown=nonroot:nonroot /app/Cargo.toml /app/

COPY --from=builder --chown=nonroot:nonroot /app/.env /app/

# Set environment variables
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:80"
ENV LEPTOS_SITE_ROOT="site"

# Use non-privileged port
EXPOSE 80

# Set working directory
WORKDIR /app

# Run as non-root user
USER nonroot

# Run the server
CMD ["/app/cosmic-rust"]
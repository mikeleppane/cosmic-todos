# If you’re using stable, use this instead
FROM rust:1.87 AS builder

# Install cargo-binstall, which makes it easier to install other
# cargo extensions like cargo-leptos
RUN wget -q https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz \
    && tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz \
    && cp cargo-binstall /usr/local/cargo/bin

# install mold linker, which is a faster linker


# Install required tools
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends clang curl mold

# Install Node.js
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
RUN wget -qO- https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install --no-install-recommends -y nodejs \
    && node -v

RUN cargo binstall cargo-leptos -y \
    && rustup target add wasm32-unknown-unknown


# Make an /app dir, which everything will eventually live in
RUN mkdir -p /app
WORKDIR /app
COPY . .

# Build the app
RUN cargo leptos build --release -vv

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

# Copy the server binary to the /app directory
COPY --from=builder /app/target/release/cosmic-rust /app/

# /target/site contains our JS/WASM/CSS, etc.
COPY --from=builder /app/target/site /app/site

# Copy Cargo.toml if it’s needed at runtime
COPY --from=builder /app/Cargo.toml /app/

# Set any required env variables and
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:80"
ENV LEPTOS_SITE_ROOT="site"
EXPOSE 80

# -- NB: update binary name from "leptos_start" to match your app name in Cargo.toml --
# Run the server
CMD ["/app/cosmic-rust"]

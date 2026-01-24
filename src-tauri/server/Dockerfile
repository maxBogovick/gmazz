# Stage 1: Builder
FROM rust:1.84-slim-bookworm as builder

WORKDIR /usr/src/app

# Install build dependencies (needed for some crates like ring, zstd-sys)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    perl \
    make \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies only
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Remove dummy build artifacts
RUN rm -f target/release/deps/file_server*
RUN rm src/main.rs

# Copy source code
COPY . .

# Build the actual application
# SQLX_OFFLINE=true allows building without a running DB if sqlx-data.json existed,
# but since we use runtime queries mostly, it's fine.
# If we used macros strictly, we'd need `cargo sqlx prepare`.
# We switched to runtime queries mostly, so it should be fine.
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (ca-certificates for HTTPS, openssl)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create user for security
RUN groupadd -r nexus && useradd -r -g nexus nexus

# Create data directories
RUN mkdir -p /app/data/files /app/data/tmp /app/data/archives && \
    chown -R nexus:nexus /app/data

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/file_server /app/file_server

# Copy .env.example as a template (user should mount real .env or pass env vars)
COPY .env.example /app/.env.example

USER nexus

# Expose port
EXPOSE 8080

# Environment variables defaults
ENV RUST_LOG=info
ENV DATA_DIR=/app/data
ENV DB_NAME=metadata.db
ENV HOST=0.0.0.0
ENV PORT=8080

# Entrypoint
CMD ["/app/file_server"]

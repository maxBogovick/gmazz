# Build stage
FROM rust:1-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev curl unzip && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project to cache dependencies
RUN cargo new --bin gmazz-file-server
WORKDIR /app/gmazz-file-server

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Build dependencies only (this layer will be cached)
RUN cargo build --release

# Remove the dummy source and copy the actual source code
RUN rm src/*.rs
COPY . .

# Build the actual application
# We touch the main file to ensure cargo rebuilds the binary
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
# ca-certificates for HTTPS, libssl3 for openssl
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/gmazz-file-server/target/release/gmazz-file-server /usr/local/bin/server

# Create data directory
RUN mkdir -p /app/data

# Set environment variables
ENV HOST=0.0.0.0
ENV PORT=8080
ENV DATA_DIR=/app/data

# Expose the application port
EXPOSE 8080

# Run the server
CMD ["server"]
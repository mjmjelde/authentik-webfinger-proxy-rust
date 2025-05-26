# Use the official Rust image as a builder
FROM rust:latest AS builder

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false appuser

# Copy the binary from builder stage
COPY --from=builder /app/target/release/authentik-webfinger-proxy-rust /usr/local/bin/authentik-webfinger-proxy

# Change ownership and make executable
RUN chown appuser:appuser /usr/local/bin/authentik-webfinger-proxy
RUN chmod +x /usr/local/bin/authentik-webfinger-proxy

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8000

# Set environment variables
ENV RUST_LOG=info

# Run the binary
CMD ["authentik-webfinger-proxy"]
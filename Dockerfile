# Multi-stage build for Nimbus Git platform  
FROM rust:latest as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libgit2-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set up workspace
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build release binary
RUN cargo build --release --bin nimbus-web

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libgit2-1.5 \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 nimbus

# Copy binary from builder
COPY --from=builder /build/target/release/nimbus-web /usr/local/bin/nimbus-web

# Create data directories
RUN mkdir -p /data/repos /data/config && \
    chown -R nimbus:nimbus /data

USER nimbus
WORKDIR /data

# Expose ports
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the service
CMD ["nimbus-web"]
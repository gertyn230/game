# Pixel Village Server - Production Dockerfile
# Build: docker build --platform linux/arm64 -t pixel-village-server .
# Run: docker run -p 8080:8080 -v /data:/data -e DATABASE_URL=sqlite:///data/game.db pixel-village-server
FROM rust:1-slim AS build
WORKDIR /app

# Install build dependencies for SQLx
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY server ./server
COPY admin.html ./admin.html
COPY client/Cargo.toml ./client/Cargo.toml

# Create dummy client to satisfy workspace
RUN mkdir -p client/src && echo 'fn main(){}' > client/src/main.rs

# Build release binary
RUN cargo build --release -p server

# Runtime stage
FROM debian:stable-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash appuser && \
    mkdir -p /data && \
    chown -R appuser:appuser /data

# Copy binary from build stage
COPY --from=build /app/target/release/server /usr/local/bin/server
RUN chmod +x /usr/local/bin/server

# Switch to non-root user
USER appuser

# Environment variables
ENV PORT=8080
ENV DATABASE_URL=sqlite:///data/game.db?mode=rwc

# Expose port
EXPOSE 8080

# Volume for persistent data
VOLUME ["/data"]

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/bin/sh", "-c", "wget --no-verbose --tries=1 --spider http://localhost:${PORT}/api/version || exit 1"]

# Run as non-root
CMD ["server"]

# Multi-stage build for smaller final image
FROM rust:1.90-slim as builder

WORKDIR /app

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY app/Cargo.toml ./app/
COPY apps/calculator/Cargo.toml ./apps/calculator/
COPY apps/file-manager/Cargo.toml ./apps/file-manager/
COPY apps/browser/Cargo.toml ./apps/browser/
COPY compat/Cargo.toml ./compat/
COPY config/Cargo.toml ./config/
COPY core/Cargo.toml ./core/
COPY db/Cargo.toml ./db/
COPY devices/Cargo.toml ./devices/
COPY fs/Cargo.toml ./fs/
COPY gui/Cargo.toml ./gui/
COPY hal/Cargo.toml ./hal/
COPY input/Cargo.toml ./input/
COPY kernel/Cargo.toml ./kernel/
COPY llm/Cargo.toml ./llm/
COPY search/Cargo.toml ./search/
COPY services/Cargo.toml ./services/
COPY tools/Cargo.toml ./tools/

# Copy source code
COPY . .

# Build for release
RUN cargo build --locked --release --bin lucastra

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 lucastra

# Create necessary directories
RUN mkdir -p /home/lucastra/.lucastra/logs \
    /home/lucastra/.lucastra/audit \
    /home/lucastra/.lucastra/metrics \
    && chown -R lucastra:lucastra /home/lucastra

# Copy binary from builder
COPY --from=builder /app/target/release/lucastra /usr/local/bin/lucastra

# Copy default configuration
COPY docs/examples/configs/prod.json /home/lucastra/.lucastra/config.json
RUN chown lucastra:lucastra /home/lucastra/.lucastra/config.json

# Switch to app user
USER lucastra
WORKDIR /home/lucastra

# Set environment
ENV LUCASTRA_CONFIG_HOME=/home/lucastra/.lucastra
ENV RUST_LOG=info

# Health check (if applicable)
# HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
#   CMD ["/usr/local/bin/lucastra", "health"] || exit 1

# Expose ports if needed (adjust as necessary)
# EXPOSE 8080

CMD ["/usr/local/bin/lucastra"]

# ---- Build Stage ----
FROM rust:latest AS builder
WORKDIR /app

# Accept DATABASE_URL as build argument (will be provided by docker-compose)
ARG DATABASE_URL

# Cache dependencies layer: copy manifest and build deps only
COPY Cargo.toml Cargo.lock ./
# Create a temporary src directory with a stub main.rs for dependency caching
RUN mkdir src && echo "fn main() {println!(\"stub\")}" > src/main.rs
RUN cargo fetch
# Remove the stub before copying real source
RUN rm -rf src

# Copy sqlx offline mode files first (needed for compile-time verification)
COPY .sqlxrc .sqlx ./
COPY .sqlx .sqlx

# Copy full source and build
COPY . .

# Enable sqlx offline mode (prevents database connection during build)
ENV SQLX_OFFLINE=true

# Build and verify the binary exists
RUN cargo build --release --bin backend && \
    ls -la /app/target/release && \
    test -f /app/target/release/backend

# ---- Runtime Stage ----
# Use Bookworm to match newer glibc used by rust:latest build image
FROM debian:bookworm-slim
WORKDIR /app

# Install openssl and ca-certificates
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder and verify it exists
COPY --from=builder /app/target/release/backend ./backend
RUN ls -la /app && test -f /app/backend && chmod +x /app/backend
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/.env .

ENV RUST_LOG=info
EXPOSE 3000

CMD ["./backend"]   
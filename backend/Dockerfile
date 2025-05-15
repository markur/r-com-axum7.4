# ---- Build Stage ----
FROM rust:1.76 as builder
WORKDIR /app

# Install required tools for SQLx offline mode
RUN cargo install sqlx-cli --no-default-features --features postgres

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy source
COPY . .
RUN cargo build --release

# ---- Runtime Stage ----
FROM debian:bullseye-slim
WORKDIR /app

# Install openssl and ca-certificates
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder
COPY --from=builder /app/target/release/backend .
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/.env .

ENV RUST_LOG=info
EXPOSE 3000

CMD ["./backend"] 
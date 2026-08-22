# --- Build stage ---
FROM rust:latest AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --locked
RUN rm -f target/release/deps/pcfl_backend* target/release/pcfl-backend

# Build actual application
COPY . .
RUN cargo build --release --locked

# --- Runtime stage ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/pcfl-backend /app/pcfl-backend
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080

CMD ["/app/pcfl-backend"]
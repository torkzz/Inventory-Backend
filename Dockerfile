# Build stage
FROM rust:1.85-slim AS builder
WORKDIR /usr/src/inventory-backend
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
RUN useradd -m -u 1000 -s /bin/bash appuser
COPY --from=builder /usr/src/inventory-backend/target/release/inventory-backend /app/inventory-backend
RUN mkdir -p /app/uploads && chown -R appuser:appuser /app
USER appuser
EXPOSE 8080
ENV PORT=8080 HOST=0.0.0.0
ENTRYPOINT ["/app/inventory-backend"]

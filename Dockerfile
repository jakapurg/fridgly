# ---- Build stage ----
FROM rust:1.96-slim AS builder
WORKDIR /app

# Copy manifests first for better layer caching, then sources.
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations

RUN cargo build --release --bin fridgly-web

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# TLS roots for outbound HTTPS (e.g. future Open Food Facts lookups).
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/fridgly-web /usr/local/bin/fridgly-web
COPY crates/fridgly-web/static ./static

ENV BIND_ADDR=0.0.0.0:3000
ENV STATIC_DIR=/app/static
EXPOSE 3000

CMD ["fridgly-web"]

# syntax=docker/dockerfile:1
#
# Build from repository root:
#   docker build -f backend/Dockerfile -t focusflow-api .

FROM rust:1.82-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY backend/Cargo.toml backend/Cargo.lock* ./
COPY backend/api/Cargo.toml ./api/
COPY backend/core/Cargo.toml ./core/
COPY backend/ai/Cargo.toml ./ai/
COPY backend/db/Cargo.toml ./db/

COPY backend/ ./

RUN cargo build --release -p focusflow-api

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/focusflow-api /usr/local/bin/focusflow_api

WORKDIR /app

EXPOSE 10000

CMD ["focusflow_api"]

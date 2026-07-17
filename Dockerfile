# One Dockerfile, two runtime targets

FROM rust:1.95-slim-trixie AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libsqlite3-dev protobuf-compiler libprotobuf-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Dependency cache
FROM base AS deps

COPY Cargo.toml Cargo.lock ./
COPY proto/Cargo.toml proto/Cargo.toml
COPY proto/build.rs proto/build.rs
COPY services/todo-service/Cargo.toml services/todo-service/
COPY web/Cargo.toml web/

RUN mkdir -p proto/src services/todo-service/src web/src \
    && echo 'syntax = "proto3"; package cache_stub;' > proto/src/cache_stub.proto \
    && echo '' > proto/src/lib.rs \
    && echo '' > services/todo-service/src/lib.rs \
    && echo 'fn main() {}' > services/todo-service/src/main.rs \
    && echo 'fn main() {}' > web/src/main.rs

RUN cargo build --release --locked --workspace --bins

RUN rm -rf \
        target/release/deps/libproto-* target/release/deps/libtodo_service-* \
        target/release/todo-service target/release/web \
        proto/src/cache_stub.proto \
    2>/dev/null || true

# Real build
FROM deps AS builder

COPY . .

RUN cargo build --release --locked --workspace --bins

# Runtime base
FROM debian:trixie-slim AS runtime-base

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# todo-service
FROM runtime-base AS runtime-todo-service

RUN apt-get update && apt-get install -y --no-install-recommends libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /workspace/target/release/todo-service ./todo-service
RUN mkdir -p /data && touch /data/todo-service.db

ENV DATABASE_URL=/data/todo-service.db
ENV LISTEN_ADDR=0.0.0.0:50051
EXPOSE 50051
CMD ["./todo-service"]

# web (gateway)
FROM runtime-base AS runtime-web

WORKDIR /app
COPY --from=builder /workspace/target/release/web ./web

ENV WEB_LISTEN_ADDR=0.0.0.0:8080
EXPOSE 8080
CMD ["./web"]

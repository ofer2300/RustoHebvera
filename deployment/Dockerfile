# Build stage
FROM rust:1.70-slim as builder

WORKDIR /usr/src/app
COPY . .

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    cargo build --release

# Runtime stage
FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y ca-certificates libssl1.1 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/rustohebru /usr/local/bin/
COPY --from=builder /usr/src/app/config /etc/rustohebru/config

ENV RUST_LOG=info
ENV CONFIG_PATH=/etc/rustohebru/config

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["rustohebru"] 
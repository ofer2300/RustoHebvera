# Build stage
FROM rust:1.70 as builder

WORKDIR /usr/src/app
COPY . .

# Install build dependencies
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary
COPY --from=builder /usr/src/app/target/release/rusto-hebru-server .
COPY --from=builder /usr/src/app/web ./web

# Set environment variables
ENV RUST_LOG=info
ENV PORT=8080

# Expose the port
EXPOSE 8080

# Run the application
CMD ["./rusto-hebru-server"] 
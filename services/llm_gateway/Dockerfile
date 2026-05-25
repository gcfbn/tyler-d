# Build stage
FROM rust:1.95-slim-bookworm as builder

RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
# Copy everything from root (protos, services, etc.)
COPY . .

# Change to llm_gateway directory to build
WORKDIR /usr/src/app/services/llm_gateway
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/app/services/llm_gateway/target/release/llm-gateway .

CMD ["./llm-gateway"]

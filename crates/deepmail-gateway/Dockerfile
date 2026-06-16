# Build stage
FROM rust:1.78-slim as builder

# Install Zig for building the rate limiter
RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    && curl -fsSL https://ziglang.org/download/0.12.0/zig-linux-x86_64-0.12.0.tar.xz | tar -xJ \
    && mv zig-linux-x86_64-0.12.0 /usr/local/zig \
    && ln -sf /usr/local/zig/zig /usr/local/bin/zig \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Zig rate limiter source
COPY ../../zig-modules/rate-limiter ./zig-modules/rate-limiter

# Build Zig library
RUN cd zig-modules/rate-limiter && zig build -Doptimize=ReleaseFast

# Copy source code
COPY . .

# Build Rust project
ENV ZIG_RATELIMITER_LIB=/app/zig-modules/rate-limiter/zig-out/lib
RUN cargo build --release -p deepmail-gateway

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy Zig library
COPY --from=builder /app/zig-modules/rate-limiter/zig-out/lib/libzig_ratelimiter.so /usr/local/lib/
RUN ldconfig

# Copy binary
COPY --from=builder /app/target/release/deepmail-gateway /usr/local/bin/

EXPOSE 8080
CMD ["deepmail-gateway"]
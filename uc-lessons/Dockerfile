# Multi-stage build for solverforge-lessons.
FROM rust:1.95-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY static/ ./static/
COPY solver.toml ./solver.toml

# Build release binary with musl target for static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest

RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy binary from builder (musl static binary)
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/solverforge-lessons ./solverforge-lessons

# Copy static files
COPY --from=builder /build/static/ ./static/

# Copy solver config
COPY --from=builder /build/solver.toml ./solver.toml

ENV PORT=7860

# Expose the same port the container binds to by default.
EXPOSE 7860

# Run the application
CMD ["./solverforge-lessons"]

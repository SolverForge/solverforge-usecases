# Multi-stage repository-local image for solverforge-fleet.
FROM rust:1.95-alpine AS builder

RUN apk add --no-cache musl-dev
WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY static/ ./static/
COPY solver.toml ./solver.toml

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest

RUN apk add --no-cache ca-certificates
WORKDIR /app

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/solverforge-fleet ./solverforge-fleet
COPY --from=builder /build/static/ ./static/
COPY --from=builder /build/solver.toml ./solver.toml

ENV PORT=7860
EXPOSE 7860

CMD ["./solverforge-fleet"]

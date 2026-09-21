FROM rust:1.95-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin solverforge-flightcrew

FROM debian:bookworm-slim
RUN useradd --create-home --uid 1000 app
WORKDIR /app
COPY --from=builder /app/target/release/solverforge-flightcrew /usr/local/bin/solverforge-flightcrew
COPY --from=builder /app/static ./static
USER app
ENV PORT=7860
EXPOSE 7860
CMD ["solverforge-flightcrew"]

# Multi-stage build for solverforge-ev. The build context is this directory:
#   docker build -f Dockerfile -t solverforge-ev .
# The image ships no road data: the first solve downloads the OpenStreetMap
# network around the plan and caches it under /app/.osm_cache.
FROM rust:1.95-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin solverforge_ev

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 app
WORKDIR /app
COPY --from=builder /app/target/release/solverforge_ev /usr/local/bin/solverforge_ev
COPY --from=builder /app/static ./static
RUN chown app:app /app
USER app
ENV PORT=7860
EXPOSE 7860
CMD ["solverforge_ev"]

FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /usr/sbin/nologin appuser

WORKDIR /app

COPY --from=builder --chown=appuser:appuser /app/target/release/device-lover-api ./device-lover-api
COPY --from=builder --chown=appuser:appuser /app/migrations ./migrations

ENV HOST=0.0.0.0
ENV PORT=3000

USER appuser

EXPOSE 3000

CMD ["./device-lover-api"]

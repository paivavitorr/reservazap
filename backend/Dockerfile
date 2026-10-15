# ─────────────────────────── build ───────────────────────────
FROM rust:1.98-bookworm AS build
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY backend backend
COPY migration migration

RUN cargo build --release -p reservazap

# ─────────────────────────── runtime ───────────────────────────
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=build /app/target/release/reservazap-cli /usr/local/bin/reservazap-cli
COPY backend/config config

ENV LOCO_ENV=production
EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -fsS http://localhost:3000/_ping || exit 1

# sobe servidor + scheduler (expiração de holds e lembretes)
CMD ["reservazap-cli", "start", "--scheduler"]

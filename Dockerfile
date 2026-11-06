# Arrhabon API: build the Rust workspace, run the arrhabon-api binary as a non-root user.
# No keys are baked in: mount them at runtime (see README "Docker").

FROM rust:1.99-slim-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p arrhabon-api

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home-dir /app arrhabon
WORKDIR /app
COPY --from=builder /src/target/release/arrhabon-api /usr/local/bin/arrhabon-api
USER arrhabon
ENV HOST=0.0.0.0 PORT=8080
EXPOSE 8080
CMD ["arrhabon-api"]

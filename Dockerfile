FROM rust:1.97-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.toml
COPY src src
COPY static static
COPY migrations migrations
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 marketshield
WORKDIR /app
COPY --from=builder /app/target/release/marketshield /usr/local/bin/marketshield
COPY static static
RUN mkdir -p /app/data && chown -R marketshield:marketshield /app
USER marketshield
ENV MARKETSHIELD_BIND=0.0.0.0:8080
ENV MARKETSHIELD_DATABASE_URL=sqlite://data/marketshield.db
EXPOSE 8080
ENTRYPOINT ["marketshield"]

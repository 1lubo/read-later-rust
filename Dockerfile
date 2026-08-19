# Multi-stage build (final deploy milestone).
#
# Java/Spring: stage 1 is `mvn package` in a full JDK image; stage 2 is copying
# the fat jar into a slim JRE. Here stage 1 is a full Rust toolchain producing a
# static-ish release binary; stage 2 is a minimal Debian with just the CA certs
# the outbound HTTPS fetcher needs.

# ---- build ----
FROM rust:1-slim-bookworm AS build
WORKDIR /app

# Cache dependencies: copy manifests, build a dummy target, then the real src.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs \
    && cargo build --release || true
RUN rm -rf src

COPY . .
RUN cargo build --release --bin readlater

# ---- runtime ----
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=build /app/target/release/readlater /usr/local/bin/readlater

# SQLite lives on the mounted Fly volume; see fly.toml.
ENV DATABASE_URL="sqlite:/data/bookmarks.db?mode=rwc"
ENV BIND_ADDR="0.0.0.0:8080"
EXPOSE 8080

CMD ["readlater"]

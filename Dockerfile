# syntax=docker/dockerfile:1

# ---------------- Builder ----------------
# Needs libfontconfig1-dev: RustQuant -> plotly -> yeslogic-fontconfig-sys
# links against the system fontconfig at build time.
FROM rust:1-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libfontconfig1-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Manifests + sources first for better layer caching.
COPY Cargo.toml Cargo.lock Toasty.toml ./
COPY src ./src
COPY toasty ./toasty

# Plain `cargo build` (no BuildKit cache mounts) so the image builds on any
# host, including ones without the buildx plugin. If your host has BuildKit,
# rebuilds can be sped up by adding
#   --mount=type=cache,target=/usr/local/cargo/registry
#   --mount=type=cache,target=/app/target
# to the RUN line below (and `cp`ing the binary out of the mounted target).
RUN cargo build --release --bin stock-simulation-engine \
    && cp target/release/stock-simulation-engine /app/server

# ---------------- Runtime ----------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libfontconfig1 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 10001 appuser

COPY --from=builder /app/server /usr/local/bin/server

USER appuser

EXPOSE 3000

ENV RUST_LOG=info

CMD ["/usr/local/bin/server"]

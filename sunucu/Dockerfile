# syntax=docker/dockerfile:1.7
#
# One Dockerfile for all server binaries: build with
#   docker build --build-arg BIN=api|engine-service|worker|ingest \
#     --build-context proto=../sozlesme/proto sunucu
# The build context is sunucu/; the gRPC contract (sozlesme/proto) is passed as
# the named context "proto" because engine-service/build.rs compiles it.
#
# Optional build secret "build_ca": a PEM CA bundle for networks behind a
# TLS-intercepting proxy (cargo trusts it via CARGO_HTTP_CAINFO). Empty/absent
# means the image's default trust store is used.

ARG RUST_IMAGE=rust:1.97.0-slim-trixie
ARG RUNTIME_IMAGE=debian:trixie-slim
ARG BIN=api

# ---- cargo-chef: dependency layer cache -------------------------------------
FROM ${RUST_IMAGE} AS chef
# The image ships the compiler pinned in sunucu/rust-toolchain.toml (1.97.0).
# Overriding
# rust-toolchain.toml stops rustup from downloading a second toolchain
# (rustfmt/clippy are not needed for release builds).
ENV RUSTUP_TOOLCHAIN=1.97.0
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=secret,id=build_ca \
    ca=/run/secrets/build_ca; if [ -s "$ca" ]; then export CARGO_HTTP_CAINFO="$ca"; fi; \
    cargo install cargo-chef --locked --version 0.1.78
WORKDIR /build/sunucu

FROM chef AS planner
COPY . .
COPY --from=proto . /build/sozlesme/proto
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/sunucu/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=secret,id=build_ca \
    ca=/run/secrets/build_ca; if [ -s "$ca" ]; then export CARGO_HTTP_CAINFO="$ca"; fi; \
    cargo chef cook --release --locked --recipe-path recipe.json
# The dependency layer above is identical for every BIN (whole workspace), so
# BuildKit builds it once; BIN only affects the steps below.
ARG BIN
COPY . .
COPY --from=proto . /build/sozlesme/proto
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=secret,id=build_ca \
    ca=/run/secrets/build_ca; if [ -s "$ca" ]; then export CARGO_HTTP_CAINFO="$ca"; fi; \
    cargo build --release --locked --bin "${BIN}" \
    && install -D "target/release/${BIN}" /out/service

# ---- grpc_health_probe (used only in the engine-service image) --------------
FROM ${RUNTIME_IMAGE} AS probe
ARG TARGETARCH
ARG GRPC_HEALTH_PROBE_VERSION=v0.4.40
ADD --chmod=0755 \
  https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-${TARGETARCH} \
  /grpc_health_probe

# ---- runtime ----------------------------------------------------------------
FROM ${RUNTIME_IMAGE} AS runtime-base
# debian-slim (not distroless) on purpose: bash gives a dependency-free
# container health check (/dev/tcp) for the HTTP services.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin app
COPY --from=builder /out/service /usr/local/bin/service
USER 10001:10001
ENTRYPOINT ["/usr/local/bin/service"]

FROM runtime-base AS final-api
FROM runtime-base AS final-worker
FROM runtime-base AS final-ingest
FROM runtime-base AS final-engine-service
COPY --from=probe /grpc_health_probe /usr/local/bin/grpc_health_probe

ARG BIN
FROM final-${BIN}

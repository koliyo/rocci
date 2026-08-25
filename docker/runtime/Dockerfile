# syntax=docker/dockerfile:1
# Content-agnostic Rocci/Rocdown runtime image (builder/dev toolchain).
# CDN hosting uses docker/cdn/Dockerfile via compose.
# Build from the repository root:
#   docker compose -f docker/compose.yml build
# Run a mounted hybrid site:
#   ./docker/docker-serve-site.sh examples/rocdown/counter

ARG UBUNTU_IMAGE=ubuntu:24.04

FROM ${UBUNTU_IMAGE} AS builder
ARG TARGETARCH
ARG ROC_NIGHTLY_DATE=2026-08-23
ARG ROC_NIGHTLY_SHA=fb208ba
ARG ROC_NIGHTLY_TAG=nightly-2026-08-23-fb208ba
ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH="/usr/local/cargo/bin:/usr/local/bin:/opt/roc:${PATH}" \
    ROCCI_CACHE=/var/cache/rocci \
    HOME=/root

COPY docker/install-linux-deps.sh docker/install-roc.sh /tmp/docker/
RUN chmod +x /tmp/docker/*.sh \
    && /tmp/docker/install-linux-deps.sh builder \
    && TARGETARCH="${TARGETARCH}" ROC_NIGHTLY_DATE="${ROC_NIGHTLY_DATE}" \
       ROC_NIGHTLY_SHA="${ROC_NIGHTLY_SHA}" ROC_NIGHTLY_TAG="${ROC_NIGHTLY_TAG}" \
       /tmp/docker/install-roc.sh \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
       | sh -s -- -y --default-toolchain stable --profile minimal \
    && mkdir -p "${ROCCI_CACHE}" /root/.rocci /root/.cache \
    && touch /root/.rocci/.keep /root/.cache/.keep

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY playground playground
RUN cargo build --release -p rocci-cli -p rocci-rocdown-cli \
    && cp target/release/rocci target/release/rocdown /usr/local/bin/ \
    && rm -rf target /usr/local/cargo/registry /usr/local/cargo/git /usr/local/rustup

FROM ${UBUNTU_IMAGE} AS runtime
ARG TARGETARCH
ARG ROC_NIGHTLY_DATE=2026-08-23
ARG ROC_NIGHTLY_SHA=fb208ba
ARG ROC_NIGHTLY_TAG=nightly-2026-08-23-fb208ba
ENV DEBIAN_FRONTEND=noninteractive \
    PATH="/usr/local/bin:/opt/roc:${PATH}" \
    ROCCI_CACHE=/var/cache/rocci \
    HOME=/root \
    ROC_BASIC_WEBSERVER_HOST=0.0.0.0 \
    ROC_BASIC_WEBSERVER_PORT=8001 \
    DB_PATH=/var/lib/rocci/site.db
WORKDIR /src/site

COPY docker/install-linux-deps.sh docker/install-roc.sh /tmp/docker/
RUN chmod +x /tmp/docker/*.sh \
    && /tmp/docker/install-linux-deps.sh runtime \
    && TARGETARCH="${TARGETARCH}" ROC_NIGHTLY_DATE="${ROC_NIGHTLY_DATE}" \
       ROC_NIGHTLY_SHA="${ROC_NIGHTLY_SHA}" ROC_NIGHTLY_TAG="${ROC_NIGHTLY_TAG}" \
       /tmp/docker/install-roc.sh \
    && mkdir -p /var/lib/rocci "${ROCCI_CACHE}" /src/site \
    && rm -rf /tmp/docker

COPY --from=builder /usr/local/bin/rocci /usr/local/bin/rocci
COPY --from=builder /usr/local/bin/rocdown /usr/local/bin/rocdown
COPY --from=builder /var/cache/rocci /var/cache/rocci
COPY --from=builder /root/.rocci /root/.rocci
COPY --from=builder /root/.cache /root/.cache

EXPOSE 8001
CMD ["rocdown", "serve-islands", "/src/site", "--no-window", "--port", "8001"]

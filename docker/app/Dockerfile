# syntax=docker/dockerfile:1
# Precompiled Rocci app process. Copy a host-built musl/Linux `server` plus `assets/`.
# No rocci CLI, roc, rustc, or WebKit.
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /app/assets
COPY server /usr/local/bin/server
COPY assets/ /app/assets/
RUN chmod +x /usr/local/bin/server
WORKDIR /app
ENV ROC_BASIC_WEBSERVER_PORT=8000 \
    DB_PATH=/var/lib/rocci/app.db
EXPOSE 8000
CMD ["server"]

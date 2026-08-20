# syntax=docker/dockerfile:1
# Precompiled island process. Copy a host-built musl/Linux `islands` binary.
# No rocci, rocdown, roc, rustc, or WebKit.
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /app/assets
COPY islands /usr/local/bin/islands
RUN chmod +x /usr/local/bin/islands
WORKDIR /app
ENV ROC_BASIC_WEBSERVER_HOST=0.0.0.0 \
    ROC_BASIC_WEBSERVER_PORT=8001 \
    DB_PATH=/var/lib/rocci/site.db
EXPOSE 8001
CMD ["islands"]

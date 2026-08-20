# syntax=docker/dockerfile:1
# Official Caddy plus a startup banner for the host-published URL.
FROM caddy:2-alpine
COPY entrypoint.sh /usr/local/bin/rocci-cdn-entrypoint
RUN chmod +x /usr/local/bin/rocci-cdn-entrypoint
ENV ROCCI_HTTP_PORT=8080
ENTRYPOINT ["/usr/local/bin/rocci-cdn-entrypoint"]

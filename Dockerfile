FROM rust:1.97-bookworm AS server-builder
WORKDIR /workspace
COPY server server
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cd server && SQLX_OFFLINE=true cargo build --release --bin fakESB

FROM oven/bun:1.3.14 AS web-builder
WORKDIR /workspace/web
COPY web/package.json web/bunfig.toml ./
RUN bun install --minimum-release-age 604800
COPY web ./
RUN bun run generate

FROM debian:trixie-slim AS runtime-base
RUN apt-get update \
    && apt-get install -y --no-install-recommends nginx tini ca-certificates \
    && rm -f /etc/nginx/sites-enabled/default /etc/nginx/sites-available/default \
    && rm -rf /var/lib/apt/lists/*

FROM runtime-base AS prebuilt-runtime
WORKDIR /app
COPY ci-image-input/app/bin/fakESB /usr/local/bin/fakESB
COPY ci-image-input/frontend-dist/ /usr/share/nginx/html/
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 80
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/entrypoint.sh"]

FROM runtime-base AS runtime
WORKDIR /app
COPY --from=server-builder /workspace/server/target/release/fakESB /usr/local/bin/fakESB
COPY --from=web-builder /workspace/web/.output/public /usr/share/nginx/html
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 80
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/entrypoint.sh"]

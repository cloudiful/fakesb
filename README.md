# fakESB

[简体中文](README.zh-CN.md)

fakESB is a configurable XML gateway for testing ESB integrations. It parses
incoming XML requests, routes them to configured targets, or renders response
templates, and records request and response snapshots in PostgreSQL.

## Features

- XML dispatch at `POST /Esbhttp/SmartEBANK`.
- Target, routing rule, response template, and request log management.
- Generated OpenAPI document at `/api/openapi.json`.
- Static Nuxt web console and multi-architecture Docker images.

## Local development

Install Rust, PostgreSQL, and Bun. Create a local environment from
`.env.example`, then run:

```bash
cd server
export DATABASE_URL=postgresql://fakesb:change-me@127.0.0.1:5432/fakesb
cargo run --bin db_init
SQLX_OFFLINE=true cargo run
```

When changing migrations or SQL queries, initialize the database before
regenerating SQLx metadata:

```bash
cd server
export DATABASE_URL=postgresql://fakesb:change-me@127.0.0.1:5432/fakesb
cargo run --bin db_init
cargo sqlx prepare --workspace -- --all-targets
SQLX_OFFLINE=true cargo check --workspace --all-targets
```

The API listens on `127.0.0.1:3000` by default. To build the web console:

```bash
cd web
bun install --minimum-release-age 604800
bun run generate
```

The generated files are written to `web/.output/public`.

## Docker

The image contains the API and static web console. Provide a PostgreSQL URL
and keep the published port private:

```bash
docker build --target runtime -t fakesb .
docker run --rm -p 127.0.0.1:8080:80 \
  -e DATABASE_URL=postgresql://fakesb:change-me@host.docker.internal:5432/fakesb \
  fakesb
```

The `/api` management endpoints have no authentication layer. Run the service
only on a trusted network, and review target URLs and stored XML snapshots as
potentially sensitive operational data.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).

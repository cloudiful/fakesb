# fakESB server

[English](README.md) | [简体中文](README.zh-CN.md)

Actix Web backend for configurable HTTP mocking and proxying. It supports
JSON, XML, and text bodies, arbitrary request paths, response templates, and
PostgreSQL request logs.

The server exposes `/healthz`, `/api/*` management endpoints, and a fallback
handler for user-configured mock or proxy rules.

## Run

```bash
export DATABASE_URL=postgresql://fakesb:change-me@127.0.0.1:5432/fakesb
cargo run --bin db_init
cargo run --bin fakESB
```

The server listens on `127.0.0.1:3000`. It reads `FAKESB_` environment
variables and applies pending migrations on startup. The management API has no
authentication layer, so keep it on a trusted network.

When SQL or migrations change, run `db_init` first and then regenerate SQLx
metadata:

```bash
cargo run --bin db_init
cargo sqlx prepare --workspace -- --all-targets
SQLX_OFFLINE=true cargo check --workspace --all-targets
```

# fakESB server

[English](README.md) | [简体中文](README.zh-CN.md)

fakESB is an Actix Web gateway for testing ESB XML integrations. It parses the incoming `SmartEBANK` message, selects an enabled rule by service and message identifiers, then either forwards the XML to a configured target or renders an enabled response template.

The server exposes:

- `POST /Esbhttp/SmartEBANK` for XML dispatch;
- `GET /healthz` for health checks;
- `GET/POST/PUT /api/targets` for target configuration;
- `GET/POST/PUT /api/rules` for priority-ordered routing rules;
- `GET/POST/PUT /api/templates` for XML response templates;
- `GET /api/logs` and `GET /api/logs/{id}` for request logs and snapshots;
- `GET /api/openapi.json` for the generated API contract.

## Configuration

The process reads environment variables using the `FAKESB_` prefix. `DATABASE_URL` is also accepted as a local development fallback. See the repository [`.env.example`](../.env.example) for non-secret defaults.

The application uses the fresh schema in `server/migrations/0001_init.sql`: `targets`, `rules`, `response_templates`, `request_logs`, and `message_snapshots`. It never drops legacy tables. Create and switch to a new PostgreSQL database explicitly before deployment; rotate any credentials that were previously exposed in local configuration.

## Run

```bash
export DATABASE_URL=postgresql://fakesb:change-me@127.0.0.1:5432/fakesb
cargo run --bin db_init
cargo run
```

The dedicated `db_init` binary reads `DATABASE_URL` and applies all pending
migrations. The application also applies pending migrations during startup.

When migrations or SQL queries change, regenerate SQLx metadata only after the
database has been migrated:

```bash
cargo run --bin db_init
cargo sqlx prepare --workspace -- --all-targets
SQLX_OFFLINE=true cargo check --workspace --all-targets
```

The server listens on `127.0.0.1:3000` by default. The root Dockerfile builds the release image together with the static Nuxt frontend.

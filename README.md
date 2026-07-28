# fakESB

[![Release](https://github.com/cloudiful/fakesb/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/cloudiful/fakesb/actions/workflows/docker-publish.yml)
[![Latest Release](https://img.shields.io/github/v/release/cloudiful/fakesb?display_name=tag&sort=semver)](https://github.com/cloudiful/fakesb/releases)
[![License](https://img.shields.io/github/license/cloudiful/fakesb)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)

[English](README.md) | [简体中文](README.zh-CN.md)

A lightweight, configurable XML gateway for ESB integration testing, with
routing, response mocking, and request inspection.

## Features

- Forward XML requests to configurable target services.
- Route requests with service and message matching rules.
- Return configurable XML response templates without calling an upstream.
- Store request and response snapshots in PostgreSQL.
- Manage targets, rules, templates, and logs through a static web console.

## Deploy With Docker

The Docker image contains the Rust gateway, the static web console, and Nginx.
Create a local environment file and set `DATABASE_URL` to a PostgreSQL URL
reachable from the container:

```bash
cp .env.example .env
docker pull ghcr.io/cloudiful/fakesb:latest
docker run --rm --name fakesb \
  --env-file .env \
  -p 127.0.0.1:8080:80 \
  ghcr.io/cloudiful/fakesb:latest
```

Open the console at <http://127.0.0.1:8080>. The XML gateway endpoint is
`POST /Esbhttp/SmartEBANK`. Pending database migrations run at startup.

For PostgreSQL running on the host, use `host.docker.internal` instead of
`127.0.0.1` in `DATABASE_URL` on Docker Desktop.

The `/api` management endpoints have no authentication layer. Keep the
published port on a trusted network and treat configured target URLs and XML
snapshots as potentially sensitive data.

GitHub Releases provide backend binaries for Linux x86_64, Linux arm64,
Windows x64, and macOS arm64. Use the Docker image when the web console is
required.

## License

Apache License 2.0. See [LICENSE](LICENSE).

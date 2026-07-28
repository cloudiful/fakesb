# fakESB

[![Release](https://github.com/cloudiful/fakesb/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/cloudiful/fakesb/actions/workflows/docker-publish.yml)
[![License](https://img.shields.io/github/license/cloudiful/fakesb)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)

[English](README.md) | [简体中文](README.zh-CN.md)

Configurable HTTP mock and proxy for JSON, XML, and text requests.

## Features

- Match any HTTP method and path with query, header, and body matchers.
- Return templated responses or proxy requests to configured targets.
- Store request and response snapshots in PostgreSQL.
- Manage targets, rules, templates, and logs through a static web console.

## Docker

```bash
cp .env.example .env
docker pull ghcr.io/cloudiful/fakesb:latest
docker run --rm --name fakesb --env-file .env -p 127.0.0.1:8080:80 ghcr.io/cloudiful/fakesb:latest
```

Open <http://127.0.0.1:8080>. Configure a rule for the path and method that
should be mocked or proxied. Database migrations run when the server starts.

The `/api` management endpoints have no authentication layer. Keep the
published port on a trusted network and treat targets and stored snapshots as
potentially sensitive data.

Backend binaries for Linux, Windows, and macOS are published in GitHub
Releases. Use the Docker image when the web console is required.

See [LICENSE](LICENSE) for the Apache License 2.0 terms.

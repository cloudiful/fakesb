# Release

[简体中文](release.zh-CN.md)

fakESB publishes Linux GNU, Windows x64, and macOS arm64 binaries together with multi-architecture Docker images from the single GitHub Actions workflow at `.github/workflows/docker-publish.yml`.

## Prerequisites

- The GitHub repository must have GitHub Packages enabled for GHCR.
- The workflow uses the repository `GITHUB_TOKEN` with `packages: write` for GHCR and `contents: write` for GitHub Releases.
- All runtime and build dependencies are fetched from public registries or the repository itself.
- Rust release builds use sccache with a Cloudflare R2 S3-compatible cache. Configure the repository secrets `R2_ACCESS_KEY_ID` and `R2_SECRET_ACCESS_KEY` for cache access; builds still work without them, but will not use the shared cache.

The workflow builds with `SQLX_OFFLINE=true`. Before the first GitHub run, use a disposable PostgreSQL database to apply the migrations and generate `server/.sqlx`:

```bash
cd server
export DATABASE_URL=postgresql://...
cargo run --bin db_init
cargo sqlx prepare --workspace -- --all-targets
SQLX_OFFLINE=true cargo check --workspace --all-targets
```

`db_init` reads `DATABASE_URL` and runs every migration in `server/migrations`.
Keep the database used for metadata generation free of private data. Do not
commit `Cargo.lock` or `bun.lock`.

## Publish

Create and push a version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow publishes these raw binary assets:

- `fakesb-v0.1.0-x86_64-unknown-linux-gnu`
- `fakesb-v0.1.0-aarch64-unknown-linux-gnu`
- `fakesb-v0.1.0-x86_64-pc-windows-msvc.exe`
- `fakesb-v0.1.0-aarch64-apple-darwin`

Each binary has a matching `.sha256` checksum asset. Docker images are published to `ghcr.io/<owner>/<repository>` with version and `latest` tags. Both tags are multi-architecture manifests for `linux/amd64` and `linux/arm64`.

Manual runs from a branch build and validate the release inputs without publishing. A run whose ref is a `v*` tag creates the formal Release and Docker manifests.

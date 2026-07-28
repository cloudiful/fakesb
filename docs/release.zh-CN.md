# 发布

[English](release.md)

fakESB 使用唯一的 GitHub Actions workflow `.github/workflows/docker-publish.yml` 发布 Linux GNU、Windows x64、macOS arm64 二进制文件和多架构 Docker 镜像。

## 前置配置

- GitHub 仓库需要启用 GHCR（GitHub Packages）。
- workflow 使用 `GITHUB_TOKEN` 的 `packages: write` 发布 GHCR，使用 `contents: write` 创建 GitHub Release。
- 所有运行时和构建依赖均来自公开注册表或当前仓库。
- Rust 发布构建使用 sccache 和 Cloudflare R2 的 S3 兼容缓存。请在仓库 Actions Secrets 中配置 `R2_ACCESS_KEY_ID` 和 `R2_SECRET_ACCESS_KEY`；未配置时仍可构建，但不会使用共享缓存。

workflow 使用 `SQLX_OFFLINE=true` 构建。首次运行前，应使用一次性的 PostgreSQL 数据库执行 migration，并生成 `server/.sqlx`：

```bash
cd server
export DATABASE_URL=postgresql://...
cargo run --bin db_init
cargo sqlx prepare --workspace -- --all-targets
SQLX_OFFLINE=true cargo check --workspace --all-targets
```

`db_init` 读取 `DATABASE_URL`，执行 `server/migrations` 中的全部 migration。
生成元数据时请使用不含私有数据的数据库。不要提交 `Cargo.lock` 或
`bun.lock`。

## 发布

创建并推送版本 tag：

```bash
git tag v0.1.0
git push origin v0.1.0
```

workflow 会直接发布以下裸二进制文件：

- `fakesb-v0.1.0-x86_64-unknown-linux-gnu`
- `fakesb-v0.1.0-aarch64-unknown-linux-gnu`
- `fakesb-v0.1.0-x86_64-pc-windows-msvc.exe`
- `fakesb-v0.1.0-aarch64-apple-darwin`

每个二进制文件都有对应的 `.sha256` 校验文件。Docker 镜像发布到 `ghcr.io/<owner>/<repository>`，同时生成版本 tag 和 `latest` tag；两个 tag 都是同时包含 `linux/amd64` 与 `linux/arm64` 的多架构 manifest。

从分支手动触发只构建并验证发布输入，不会发布。ref 为 `v*` 版本 tag 时，才会创建正式 Release 和 Docker manifest。

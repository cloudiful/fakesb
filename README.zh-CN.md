# fakESB

[English](README.md)

fakESB 是一个可配置的 XML 网关，用于测试 ESB 集成。它解析输入 XML
请求，将请求转发到配置的目标地址，或渲染响应模板，并将请求与响应快照保存到 PostgreSQL。

## 功能

- `POST /Esbhttp/SmartEBANK` XML 转发接口。
- 目标地址、路由规则、响应模板和请求日志管理。
- `/api/openapi.json` 自动生成的 OpenAPI 文档。
- Nuxt 静态管理界面和多架构 Docker 镜像。

## 本地开发

安装 Rust、PostgreSQL 和 Bun。根据 `.env.example` 创建本地环境，然后运行：

```bash
cd server
export DATABASE_URL=postgresql://fakesb:change-me@127.0.0.1:5432/fakesb
cargo run --bin db_init
SQLX_OFFLINE=true cargo run
```

修改 migration 或 SQL 查询时，应先初始化数据库，再重新生成 SQLx 元数据：

```bash
cd server
export DATABASE_URL=postgresql://fakesb:change-me@127.0.0.1:5432/fakesb
cargo run --bin db_init
cargo sqlx prepare --workspace -- --all-targets
SQLX_OFFLINE=true cargo check --workspace --all-targets
```

API 默认监听 `127.0.0.1:3000`。构建前端管理界面：

```bash
cd web
bun install --minimum-release-age 604800
bun run generate
```

生成文件位于 `web/.output/public`。

## Docker

镜像包含 API 和静态前端。配置 PostgreSQL 地址，并将发布端口限制在可信网络：

```bash
docker build --target runtime -t fakesb .
docker run --rm -p 127.0.0.1:8080:80 \
  -e DATABASE_URL=postgresql://fakesb:change-me@host.docker.internal:5432/fakesb \
  fakesb
```

`/api` 管理接口当前没有认证层。服务只能运行在可信网络中；目标地址和保存的 XML
快照应按可能包含敏感运行数据处理。

## 许可证

本项目采用 Apache License 2.0，详见 [LICENSE](LICENSE)。

# fakESB 服务端

[English](README.md) | [简体中文](README.zh-CN.md)

fakESB 是用于测试 ESB XML 集成的 Actix Web 网关。服务解析 `SmartEBANK` 请求，根据服务号和报文标识选择启用规则，然后将报文透传到目标地址或渲染启用的响应模板。

服务提供：

- `POST /Esbhttp/SmartEBANK` XML 转发接口；
- `GET /healthz` 健康检查；
- `GET/POST/PUT /api/targets` 目标地址管理；
- `GET/POST/PUT /api/rules` 按优先级管理路由规则；
- `GET/POST/PUT /api/templates` XML 响应模板管理；
- `GET /api/logs` 和 `GET /api/logs/{id}` 请求日志及报文快照；
- `GET /api/openapi.json` API 契约。

## 配置

进程读取 `FAKESB_` 前缀的环境变量。本地开发也接受 `DATABASE_URL` 作为数据库地址。无凭据示例见仓库根目录的 `.env.example`。

服务使用 `server/migrations/0001_init.sql` 中的新数据库模型：`targets`、`rules`、`response_templates`、`request_logs` 和 `message_snapshots`。它不会删除旧表。部署前请明确创建并切换到新的 PostgreSQL 数据库，并轮换曾经暴露在本地配置中的凭据。

## 运行

```bash
export DATABASE_URL=postgresql://fakesb:change-me@127.0.0.1:5432/fakesb
cargo run --bin db_init
cargo run
```

专用的 `db_init` binary 读取 `DATABASE_URL` 并执行所有待处理的
migration；应用启动时也会执行待处理的 migration。

修改 migration 或 SQL 查询时，必须在数据库完成迁移后再重新生成 SQLx 元数据：

```bash
cargo run --bin db_init
cargo sqlx prepare --workspace -- --all-targets
SQLX_OFFLINE=true cargo check --workspace --all-targets
```

默认监听 `127.0.0.1:3000`。根目录 Dockerfile 会将 Rust 服务和 Nuxt 静态前端构建到同一个镜像。

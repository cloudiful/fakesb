# fakESB 服务端

[English](README.md) | [简体中文](README.zh-CN.md)

基于 Actix Web 的可配置 HTTP Mock 与代理后端，支持 JSON、XML、文本正文、任意请求
路径、响应模板以及 PostgreSQL 请求日志。

服务提供 `/healthz`、`/api/*` 管理接口，以及由用户规则驱动的通用 fallback 处理器。

## 运行

```bash
export DATABASE_URL=postgresql://fakesb:change-me@127.0.0.1:5432/fakesb
cargo run --bin db_init
cargo run --bin fakESB
```

默认监听 `127.0.0.1:3000`，读取 `FAKESB_` 前缀环境变量，并在启动时执行待处理的
migration。管理接口没有认证层，请只在可信网络中使用。

修改 SQL 或 migration 后，先迁移数据库，再生成 SQLx 元数据：

```bash
cargo run --bin db_init
cargo sqlx prepare --workspace -- --all-targets
SQLX_OFFLINE=true cargo check --workspace --all-targets
```

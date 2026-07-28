# fakESB 前端

[English](README.md) | [简体中文](README.zh-CN.md)

管理台是使用 Nuxt 和 Nuxt UI 构建的静态应用，提供目标地址、路由规则、响应模板、请求日志和日志详情页面。默认语言为简体中文，也可在页面顶部切换英文。

浏览器使用生成的 OpenAPI 类型和 `openapi-fetch` 客户端。生产环境通过同源 `/api` 路由请求后端，静态生成阶段不会访问 PostgreSQL。

## 本地开发

```bash
bun install --minimum-release-age 604800
bun run dev
```

Nuxt 单独运行时，可将 `NUXT_PUBLIC_API_BASE` 设置为 Rust 服务地址，例如 `http://127.0.0.1:3000`。生产环境默认使用同源地址。

## 静态构建

```bash
bun run generate
```

可部署文件位于 `.output/public`。Rust OpenAPI 注解变化后，执行 `bun run api:generate` 刷新前端契约。

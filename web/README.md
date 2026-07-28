# fakESB web

[English](README.md) | [简体中文](README.zh-CN.md)

The web console is a static Nuxt application using Nuxt UI. It provides pages for target addresses, routing rules, response templates, request logs, and log details. The default locale is Simplified Chinese; English is available from the header.

The browser uses the generated OpenAPI types and `openapi-fetch` client. Production requests use the same-origin `/api` routes and do not access PostgreSQL during static generation.

## Local development

```bash
bun install --minimum-release-age 604800
bun run dev
```

Set `NUXT_PUBLIC_API_BASE` to the Rust server origin when the Nuxt dev server runs separately, for example `http://127.0.0.1:3000`. The production default is same-origin.

## Static build

```bash
bun run generate
```

The deployable files are written to `.output/public`. Refresh the client contract with `bun run api:generate` after the Rust OpenAPI annotations change.

# fakESB

[![Release](https://github.com/cloudiful/fakesb/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/cloudiful/fakesb/actions/workflows/docker-publish.yml)
[![Latest Release](https://img.shields.io/github/v/release/cloudiful/fakesb?display_name=tag&sort=semver)](https://github.com/cloudiful/fakesb/releases)
[![License](https://img.shields.io/github/license/cloudiful/fakesb)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)

[English](README.md) | [简体中文](README.zh-CN.md)

一个轻量、可配置的 XML 网关，用于 ESB 集成测试，支持路由、响应模拟和请求检查。

## 功能

- 将 XML 请求转发到可配置的目标服务。
- 按服务标识和报文标识匹配路由规则。
- 使用 XML 响应模板直接返回模拟结果，无需调用上游服务。
- 将请求和响应快照保存到 PostgreSQL。
- 通过静态 Web 管理台管理目标地址、规则、模板和日志。

## Docker 部署

Docker 镜像包含 Rust 网关、静态 Web 管理台和 Nginx。创建本地环境文件，
并将 `DATABASE_URL` 设置为容器可以访问的 PostgreSQL 地址：

```bash
cp .env.example .env
docker pull ghcr.io/cloudiful/fakesb:latest
docker run --rm --name fakesb \
  --env-file .env \
  -p 127.0.0.1:8080:80 \
  ghcr.io/cloudiful/fakesb:latest
```

打开管理台：<http://127.0.0.1:8080>。XML 网关接口为
`POST /Esbhttp/SmartEBANK`。服务启动时会自动执行待处理的数据库 migration。

如果 PostgreSQL 运行在宿主机上，请在 Docker Desktop 环境中将
`DATABASE_URL` 里的 `127.0.0.1` 改为 `host.docker.internal`。

`/api` 管理接口当前没有认证层。请只在可信网络中发布端口，并将配置的目标地址
和 XML 快照按可能包含敏感数据处理。

GitHub Releases 提供 Linux x86_64、Linux arm64、Windows x64 和 macOS arm64
的后端二进制。需要 Web 管理台时，请使用 Docker 镜像。

## 许可证

Apache License 2.0，详见 [LICENSE](LICENSE)。

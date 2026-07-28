# fakESB

[![Release](https://github.com/cloudiful/fakesb/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/cloudiful/fakesb/actions/workflows/docker-publish.yml)
[![许可证](https://img.shields.io/github/license/cloudiful/fakesb)](LICENSE)

[English](README.md) | [简体中文](README.zh-CN.md)

可配置的 HTTP Mock 与代理服务，支持 JSON、XML 和文本请求。

## 功能

- 按任意 HTTP 方法和路径匹配，并支持查询参数、请求头和正文匹配。
- 返回模板化响应，或将请求转发到配置的目标地址。
- 使用 PostgreSQL 保存请求、响应和快照。
- 通过静态 Web 管理台管理目标、规则、模板和日志。

## Docker 部署

```bash
cp .env.example .env
docker pull ghcr.io/cloudiful/fakesb:latest
docker run --rm --name fakesb --env-file .env -p 127.0.0.1:8080:80 ghcr.io/cloudiful/fakesb:latest
```

打开 <http://127.0.0.1:8080>，为需要 Mock 或代理的路径和方法配置规则。
服务启动时会自动执行数据库 migration。

`/api` 管理接口没有认证层。请只在可信网络中发布端口，并将目标地址及保存的
快照按可能包含敏感数据处理。

Linux、Windows 和 macOS 后端二进制发布在 GitHub Releases；需要 Web 管理台时使用
Docker 镜像。

许可证为 Apache-2.0，详见 [LICENSE](LICENSE)。

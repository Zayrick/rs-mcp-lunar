# Cloudflare Worker 编译与部署

本项目只提供 Cloudflare Worker 运行方案。Rust 代码由 `worker-build` 编译为 WebAssembly 和 ES module；编译发生在 GitHub Actions，Wrangler 随后上传同一 job 中生成的 `build/`，Cloudflare 端不执行 Rust 编译。

## GitHub Actions 自动部署

`.github/workflows/deploy.yml` 在以下情况运行：

- push 到 `main`；
- 在 GitHub Actions 页面手动触发 `workflow_dispatch`。

工作流依次完成：

1. 安装 Rust、`wasm32-unknown-unknown`、Node.js 和锁定依赖；
2. 安装固定版本 `worker-build 0.8.5`；
3. 执行 `npm run build`，生成完整 Worker bundle；
4. 执行 `npm run deploy`，让 Wrangler 上传已生成的 `build/index.js` 和 Wasm 模块。

`wrangler.jsonc` 没有 `build.command`，因此部署阶段不会触发第二次构建，也不会把编译工作转交给 Cloudflare Workers Builds。

在 GitHub 仓库的 `Settings → Environments → production` 中配置：

| Secret | 用途 |
| --- | --- |
| `CLOUDFLARE_API_TOKEN` | 部署 Worker 的 API Token |
| `CLOUDFLARE_ACCOUNT_ID` | 目标 Cloudflare 账户 ID |

API Token 应限制到目标账户，并只授予 Worker 部署所需权限。不要把 Token 写入仓库或工作流参数。

## 本地构建与运行

需要 Rust 1.89+、Node.js 22+ 和 npm：

```sh
rustup target add wasm32-unknown-unknown
cargo install worker-build --version 0.8.5 --locked
npm ci
npm run build
```

产物位于：

```text
build/
├── index.js
├── index_bg.wasm
├── package.json
└── worker/shim.mjs
```

`build/` 是生成物，不提交到 Git。启动本地 Worker：

```sh
npm run dev
```

默认 MCP URL 是 `http://127.0.0.1:8787/lunar`。`npm run dev` 会先重新构建，避免运行过期产物。

## 手动预检与部署

手动部署也必须先显式构建：

```sh
npm run build
npm run deploy:dry-run
npm run deploy
```

`deploy` 和 `deploy:dry-run` 都不会自行编译。如果 `build/index.js` 不存在，Wrangler 会直接失败，这能保证编译边界清晰，不会静默切回 Cloudflare 内构建。

正式部署后 Wrangler 会打印 Worker 地址，MCP URL 为该地址加精确路径 `/lunar`。

## 验证 MCP

```sh
curl -N http://127.0.0.1:8787/lunar \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  --data '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"curl","version":"1"}}}'
```

路径必须是 `/lunar`；`/lunar/` 是信息页。Streamable HTTP 客户端应在 `Accept` 中同时包含 `application/json` 和 `text/event-stream`。

## 浏览器 Origin

命令行 MCP 客户端通常不发送 `Origin`。浏览器客户端必须出现在 `MCP_ALLOWED_ORIGINS` 中，例如：

```jsonc
{
  "vars": {
    "MCP_ALLOWED_ORIGINS": "https://app.example.com,https://admin.example.com"
  }
}
```

匹配包含 scheme、主机和端口，必须完全一致。通配符 `*` 和 `Origin: null` 会被拒绝。这个变量不是认证机制。

## 自定义域名（可选）

域名已托管在同一 Cloudflare 账户时，可在 `wrangler.jsonc` 中关闭 `workers_dev` 并添加 Custom Domain：

```jsonc
{
  "workers_dev": false,
  "routes": [
    {"pattern": "mcp.example.com", "custom_domain": true}
  ]
}
```

最终 MCP URL 为 `https://mcp.example.com/lunar`。

## 上线前检查

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --target wasm32-unknown-unknown --lib --locked -- -D warnings
cargo test --all-targets --locked
npm run build
npm run deploy:dry-run
npm run check:startup
```

Worker 自身限制请求体为 1 MiB，并执行严格 Origin 校验；生产环境仍应配置 Cloudflare Access（私有服务）、WAF、Rate Limiting 和日志监控。更多约束见 [`SECURITY.md`](../SECURITY.md)。

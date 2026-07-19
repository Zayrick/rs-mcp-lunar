# Cloudflare Worker 编译与部署

本项目已经可以从同一套 Rust 业务代码生成原生服务器和 Cloudflare Worker。Worker 目标不会链接 Axum、Tokio 网络栈或 rmcp 的原生 HTTP 服务，而是使用 workers-rs 的 `fetch` 入口；十个工具、输入校验与 Markdown 输出仍与原生目标共用。

Cloudflare 当前的 Rust 流程是使用 `worker-build` 将 `wasm32-unknown-unknown` 产物、wasm-bindgen JavaScript glue 和优化步骤打包成 ES module Worker。部署入口是 `build/index.js`，不是把裸 `.wasm` 文件填入旧版 `type = "rust"` 配置。参见 Cloudflare 的 [Rust Workers 指南](https://developers.cloudflare.com/workers/languages/rust/)、[workers-rs](https://github.com/cloudflare/workers-rs) 和 [Wrangler custom builds](https://developers.cloudflare.com/workers/wrangler/custom-builds/)。

## 1. 准备环境

需要：

- Rust 1.89 或更新版本。
- Node.js 22 或更新版本；仓库锁定的 Wrangler 4.112.0 要求 Node 22+。
- npm 和一个 Cloudflare 账户。

在仓库根目录执行：

```sh
rustup target add wasm32-unknown-unknown
npm ci
```

`npm ci` 会严格使用 `package-lock.json` 安装 Wrangler。`npm run build:worker` 会按固定版本安装 `worker-build 0.8.5`，无需全局安装 Wrangler。

## 2. 编译 Worker

```sh
npm run build:worker
```

成功后生成：

```text
build/
├── index.js
├── index_bg.wasm
├── package.json
└── worker/shim.mjs
```

也可以用下面的命令只检查 Rust Wasm 目标；它不会生成可直接部署的 JavaScript glue，因此正式构建仍以 `npm run build:worker` 为准。

```sh
cargo build --release --target wasm32-unknown-unknown --locked
```

不要把 `[profile.release]` 改为 `strip = true`。worker-build 0.8 在 wasm-bindgen 阶段仍需要 externref transform 元数据；项目已使用 `strip = "debuginfo"` 并由后续 wasm-opt 完成最终优化。

## 3. 本地运行和验证

启动 Cloudflare 的本地 Worker 运行时：

```sh
npm run dev
```

默认 MCP URL 是 `http://127.0.0.1:8787/lunar`。另开终端发送初始化请求：

```sh
curl -N http://127.0.0.1:8787/lunar \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  --data '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"curl","version":"1"}}}'
```

响应应为 `200`、`Content-Type: text/event-stream`，其中 `data:` 事件包含 `Lunar Calendar MCP`。随后可列出工具：

```sh
curl -N http://127.0.0.1:8787/lunar \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'MCP-Protocol-Version: 2025-11-25' \
  --data '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
```

注意：路径必须是 `/lunar`；`/lunar/` 是信息页，不是 MCP 端点。Streamable HTTP 客户端应在 `Accept` 中同时声明 `application/json` 和 `text/event-stream`。

## 4. 配置浏览器 Origin

命令行 MCP 客户端通常不发送 `Origin`，无需额外配置。浏览器客户端会发送 `Origin`，Worker 默认拒绝它，防止任意网页调用公开 MCP 服务。

本地测试单一来源：

```sh
npx wrangler dev --var MCP_ALLOWED_ORIGINS:https://app.example.com
```

生产环境可在 `wrangler.jsonc` 顶层加入普通变量：

```jsonc
{
  "vars": {
    "MCP_ALLOWED_ORIGINS": "https://app.example.com,https://admin.example.com"
  }
}
```

匹配包含 scheme、主机和端口，必须完全一致；逗号分隔多个来源。通配符 `*` 和 `Origin: null` 会被拒绝。这个变量不是凭据，不要用它代替身份认证。

## 5. 登录、预检和部署

Wrangler 的当前命令是 `wrangler deploy`，不是已弃用的 `wrangler publish`。

```sh
npx wrangler login
npm run deploy:dry-run
npm run deploy
```

`deploy:dry-run` 会重新执行构建、校验配置并输出上传体积，但不会修改 Cloudflare 账户。正式部署成功后 Wrangler 会打印类似下面的地址：

```text
https://rs-mcp-lunar.<你的-subdomain>.workers.dev
```

提供给 MCP 客户端的最终 URL 是：

```text
https://rs-mcp-lunar.<你的-subdomain>.workers.dev/lunar
```

本地交互登录和非交互 API Token 的官方说明见 [Wrangler authentication](https://developers.cloudflare.com/workers/wrangler/commands/general/)；部署命令见 [Wrangler Workers commands](https://developers.cloudflare.com/workers/wrangler/commands/workers/)。不要把 API Token 写进仓库。

## 6. 使用自定义域名（可选）

域名已托管在同一 Cloudflare 账户时，可将 `wrangler.jsonc` 中的 `workers_dev` 改为 `false`，并添加：

```jsonc
{
  "workers_dev": false,
  "routes": [
    {
      "pattern": "mcp.example.com",
      "custom_domain": true
    }
  ]
}
```

再次运行 `npm run deploy`。最终 MCP URL 为 `https://mcp.example.com/lunar`。Custom Domain 的 pattern 只写完整域名，不写 `/*`；Cloudflare 会管理 DNS 记录和证书。具体限制见 [Custom Domains](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/)。如果只是将 Worker 挂在现有站点的某个 URL pattern，应改用普通 [Routes](https://developers.cloudflare.com/workers/configuration/routing/routes/)。

## 7. 上线前检查

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --target wasm32-unknown-unknown --lib --locked -- -D warnings
cargo test --all-targets --locked
npm run build:worker
npm run deploy:dry-run
npm run check:startup
```

`wrangler check startup` 当前是 alpha 命令，但可以确认生成的模块能由 Workers runtime 启动。项目 CI 也会检查原生目标、Wasm 目标和完整 Worker bundle。

## 8. 生产安全

当前工具全部是本地确定性计算，不需要出站网络、KV、D1、Durable Objects 或运行时 secret；这也是 Worker 可以保持无状态的原因。但是 `workers.dev` 或 Custom Domain 部署本身是公开 HTTP 服务，并不自动提供应用认证。

上线前至少应：

- 根据流量配置 Cloudflare WAF 和 Rate Limiting；Worker 自身已将请求体限制为 1 MiB。
- 浏览器访问只允许明确的 `MCP_ALLOWED_ORIGINS`，并注意 CORS 不是认证。
- 私有服务使用 Cloudflare Access 或等价网关，并确认 MCP 客户端能携带所需凭据。
- 在 Cloudflare Workers Logs/Traces 中监控异常；仓库配置为记录调用日志并以 1% 比例采样 traces。
- 对照当前 [Workers limits](https://developers.cloudflare.com/workers/platform/limits/) 和账户套餐确认上传体积、CPU 与请求限制。

更多项目侧约束见 [`SECURITY.md`](../SECURITY.md)，MCP Streamable HTTP 要求见 [MCP transports](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)。

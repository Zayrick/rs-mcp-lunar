# rs-mcp-lunar

BaZi and Zi Wei Dou Shu MCP server for Cloudflare Workers, written in Rust and compiled to WebAssembly. It exposes a stateless Streamable HTTP endpoint at `/lunar` and returns Markdown-only tool results.

## Tools

| Domain | Tools |
| --- | --- |
| BaZi | `bazi_chart`, `bazi_structure`, `bazi_timeline`, `bazi_period_detail`, `bazi_shensha` |
| Zi Wei Dou Shu | `ziwei_chart`, `ziwei_palace_detail`, `ziwei_horoscope_overview`, `ziwei_scope_detail`, `ziwei_topic_context` |

Calendar calculations use `tyme4rs` and `iztro`; public input schemas are validated by `jsonschema`.

## Local development

Install Rust 1.89+, Node.js 22+, the Wasm target, and the pinned Worker builder:

```sh
rustup target add wasm32-unknown-unknown
cargo install worker-build --version 0.8.5 --locked
npm ci
npm run dev
```

The local MCP URL is `http://127.0.0.1:8787/lunar`. `npm run dev` builds `build/index.js` and `build/index_bg.wasm` before starting Wrangler.

## Deployment

Pushes to `main` run `.github/workflows/deploy.yml`. GitHub Actions compiles the complete Worker bundle first, then Wrangler uploads that prebuilt bundle without invoking any Cloudflare-side custom build.

Configure these GitHub Actions secrets in the `production` environment:

- `CLOUDFLARE_API_TOKEN`: scoped to deploy Workers.
- `CLOUDFLARE_ACCOUNT_ID`: the target Cloudflare account ID.

Manual deployment uses the same prebuilt-artifact boundary:

```sh
npm run build
npm run deploy:dry-run
npm run deploy
```

Browser clients that send an `Origin` header must be listed in the `MCP_ALLOWED_ORIGINS` Worker variable. See the [deployment guide](docs/CLOUDFLARE_WORKER_DEPLOYMENT.md) for setup and production notes.

## Verify

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --target wasm32-unknown-unknown --lib --locked -- -D warnings
cargo test --all-targets --locked
npm run build
npm run deploy:dry-run
```

See [architecture](docs/ARCHITECTURE.md), [contract notes](docs/COMPATIBILITY.md), [security notes](SECURITY.md), and [third-party notices](THIRD_PARTY_NOTICES.md).

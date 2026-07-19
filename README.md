# rs-mcp-lunar

BaZi and Zi Wei Dou Shu MCP server written in Rust. It runs either as a native Axum server or as a Rust/WebAssembly Cloudflare Worker. Both targets expose the same stateless Streamable HTTP endpoint at `/lunar` and return Markdown-only tool results.

## Tools

| Domain | Tools |
| --- | --- |
| BaZi | `bazi_chart`, `bazi_structure`, `bazi_timeline`, `bazi_period_detail`, `bazi_shensha` |
| Zi Wei Dou Shu | `ziwei_chart`, `ziwei_palace_detail`, `ziwei_horoscope_overview`, `ziwei_scope_detail`, `ziwei_topic_context` |

Calendar calculations are delegated to `tyme4rs` and `iztro`; public input schemas are validated by `jsonschema`.

## Native server

Rust 1.89 or newer is required.

```sh
cargo build --release --locked
```

The deployable binary is `target/release/rs-mcp-lunar`.
Build it for the same operating system and CPU architecture as the server. Deployment only requires this binary; the source tree, `Cargo.toml`, and `Cargo.lock` do not need to be copied to the server.

The server listens on `127.0.0.1:8788` by default:

```sh
RUST_LOG=info ./target/release/rs-mcp-lunar
```

Override the address with `LUNAR_MCP_ADDR`:

```sh
LUNAR_MCP_ADDR=0.0.0.0:8788 RUST_LOG=info ./target/release/rs-mcp-lunar
```

The MCP URL is `http://HOST:8788/lunar`. When listening on a public interface, place the process behind a reverse proxy that provides TLS, authentication, rate limiting, and request-size limits.

## Cloudflare Worker

The checked-in Worker toolchain uses Node.js 22 or newer, Wrangler 4.112.0, workers-rs 0.8.5, and `worker-build` 0.8.5.

```sh
rustup target add wasm32-unknown-unknown
npm ci
npm run build:worker
```

The deployable module is generated at `build/index.js` with its WebAssembly module at `build/index_bg.wasm`. Run it locally with `npm run dev`, then deploy it with:

```sh
npx wrangler login
npm run deploy:dry-run
npm run deploy
```

Wrangler prints the deployed hostname; append the exact path `/lunar` to get the MCP URL. Browser clients that send an `Origin` header must be listed in the `MCP_ALLOWED_ORIGINS` Worker variable. Command-line MCP clients that omit `Origin` continue to work.

For Cloudflare dashboard Git deployments, use `npm run build:cloudflare` as the Build command and `npm run deploy` as the Deploy command. The dashboard build script bootstraps Rust and the Wasm target because Workers Builds does not currently document Rust/Cargo as preinstalled tooling.

See the complete [Cloudflare Worker build and deployment guide](docs/CLOUDFLARE_WORKER_DEPLOYMENT.md), including local verification, Origin configuration, custom domains, and production security.

## Verify

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --target wasm32-unknown-unknown --lib --locked -- -D warnings
cargo test --all-targets --locked
npm run build:worker
npm run deploy:dry-run
```

See [architecture](docs/ARCHITECTURE.md), [contract notes](docs/COMPATIBILITY.md), [Cloudflare Worker deployment](docs/CLOUDFLARE_WORKER_DEPLOYMENT.md), [Ubuntu deployment](docs/ubuntu-deployment.html), [security notes](SECURITY.md), and [third-party notices](THIRD_PARTY_NOTICES.md).

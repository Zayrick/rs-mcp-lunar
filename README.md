# rs-mcp-lunar

BaZi and Zi Wei Dou Shu MCP server written in Rust. It provides a stateless Streamable HTTP endpoint at `/lunar` and returns Markdown-only tool results.

## Tools

| Domain | Tools |
| --- | --- |
| BaZi | `bazi_chart`, `bazi_structure`, `bazi_timeline`, `bazi_period_detail`, `bazi_shensha` |
| Zi Wei Dou Shu | `ziwei_chart`, `ziwei_palace_detail`, `ziwei_horoscope_overview`, `ziwei_scope_detail`, `ziwei_topic_context` |

Calendar calculations are delegated to `tyme4rs` and `iztro`; public input schemas are validated by `jsonschema`.

## Build

Rust 1.89 or newer is required.

```sh
cargo build --release --locked
```

The deployable binary is `target/release/rs-mcp-lunar`.
Build it for the same operating system and CPU architecture as the server. Deployment only requires this binary; the source tree, `Cargo.toml`, and `Cargo.lock` do not need to be copied to the server.

## Run

The server listens on `127.0.0.1:8788` by default:

```sh
RUST_LOG=info ./target/release/rs-mcp-lunar
```

Override the address with `LUNAR_MCP_ADDR`:

```sh
LUNAR_MCP_ADDR=0.0.0.0:8788 RUST_LOG=info ./target/release/rs-mcp-lunar
```

The MCP URL is `http://HOST:8788/lunar`. When listening on a public interface, place the process behind a reverse proxy that provides TLS, authentication, rate limiting, and request-size limits.

## Verify

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
```

See [architecture](docs/ARCHITECTURE.md), [contract notes](docs/COMPATIBILITY.md), [security notes](SECURITY.md), and [third-party notices](THIRD_PARTY_NOTICES.md).

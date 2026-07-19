# Architecture

The project has two runtime targets that share one contract, validator, and domain implementation.

```text
                       ┌─ native: Axum + rmcp + Tokio
HTTP /lunar request ──┤
                       └─ Worker: workers-rs fetch + Web API futures
                                      │
                                      ▼
                         neutral MCP/JSON-RPC dispatcher
                                      │
                                      ▼
                         contract + JSON Schema validation
                                      │
                                      ▼
                              domain dispatcher
                    ┌─────────────────┴─────────────────┐
                    ▼                                   ▼
      bazi: tyme4rs → typed facts → Markdown   ziwei: iztro → typed facts → Markdown
```

## Responsibilities

- `src/contract.rs` is the single source of truth for server identity, route, tool order, descriptions, and input schemas.
- `src/validation.rs` validates public arguments with `jsonschema`.
- `src/domain/bazi` and `src/domain/ziwei` own calendar adaptation and rendering. Calendar arithmetic stays in the upstream libraries.
- `src/mcp/protocol.rs` is runtime-neutral and owns the stateless JSON-RPC/MCP dispatch and common tool execution result.
- `src/mcp/server.rs` maps that common result into native `rmcp` types.
- `src/transport/http.rs` owns the native Axum/rmcp Streamable HTTP transport.
- `src/transport/cloudflare.rs` owns the Worker request, bounded-body, Origin, CORS, and SSE behavior without starting a Tokio runtime or socket listener.
- `src/main.rs` only configures and starts the native server. The Worker fetch entry point is in `src/lib.rs`.

Adding a tool requires one contract entry, one domain use case, one dispatch arm, and tests. Transport code does not calculate calendar facts, and domain code does not build HTTP responses.

## Why the Worker has a separate adapter

The native rmcp Streamable HTTP service starts Tokio tasks even in stateless mode. Cloudflare Workers execute Rust futures through the JavaScript event loop and do not provide a Tokio network runtime, so compiling Axum/rmcp/Tokio unchanged for `wasm32-unknown-unknown` fails in the socket stack. Target-specific dependencies keep that native stack out of the Wasm graph; the Worker adapter implements the small stateless MCP surface directly and reuses the same tool contract and domain dispatcher.

`worker-build` runs Cargo, wasm-bindgen, and Wasm optimization, producing the ES module entry point and Wasm module expected by Workers. Release builds strip debug information but deliberately do not use Cargo's boolean `strip = true`: stripping the whole Wasm before wasm-bindgen removes externref transform metadata required by worker-build 0.8.

## Dependency choices

- `tyme4rs 1.5.0` handles BaZi and sexagenary-calendar arithmetic.
- `iztro 0.9.0` handles Zi Wei natal and horoscope facts.
- `jsonschema 0.48.0` handles schema validation.
- `rmcp 2.2.0` provides the Streamable HTTP MCP server.
- `worker` and `worker-macros 0.8.5` provide the Cloudflare fetch entry point and Web API bindings.
- `worker-build 0.8.5` produces the deployable Worker bundle; Wrangler 4.112.0 runs and deploys it.

XALEN is not included because its current Chinese API does not replace the complete fortune-cycle, localized presentation, and multi-level horoscope data required by these ten tools. Adding a second calculation engine would increase maintenance without removing either existing engine.

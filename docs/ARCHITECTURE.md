# Architecture

The project has one runtime target: a Rust/WebAssembly Cloudflare Worker.

```text
Cloudflare Worker request
          │
          ▼
workers-rs fetch adapter (HTTP, Origin, CORS, bounded body, SSE)
          │
          ▼
stateless MCP/JSON-RPC dispatcher
          │
          ▼
contract + JSON Schema validation
          │
          ▼
domain dispatcher
   ┌──────┴──────┐
   ▼             ▼
 BaZi          Zi Wei
tyme4rs         iztro
   └──────┬──────┘
          ▼
       Markdown
```

## Responsibilities

- `src/lib.rs` exports the Worker fetch entry point.
- `src/transport/cloudflare.rs` implements the exact `/lunar` route, bounded request bodies, Origin validation, CORS, and Streamable HTTP responses.
- `src/mcp/protocol.rs` owns stateless MCP/JSON-RPC dispatch and tool execution results.
- `src/contract.rs` is the source of truth for server identity, route, tool order, descriptions, and schemas.
- `src/validation.rs` validates public arguments.
- `src/domain` owns calendar adaptation and Markdown rendering.

The Worker fetch adapter is the sole transport and deployment target.

## Build and deployment boundary

`worker-build` runs in GitHub Actions and produces `build/index.js`, `build/index_bg.wasm`, and their support modules. `wrangler.jsonc` points directly at `build/index.js` and deliberately has no custom `build.command`. Consequently, `wrangler deploy` only bundles and uploads the already-built artifact; Cloudflare does not install Rust or compile the project.

The deployment workflow runs build and deploy as consecutive steps in one job so the exact generated files are what Wrangler uploads.

## Dependencies

- `tyme4rs 1.5.0`: BaZi and sexagenary-calendar arithmetic.
- `iztro 0.9.0`: Zi Wei natal and horoscope facts.
- `jsonschema 0.48.0`: input validation.
- `worker` and `worker-macros 0.8.5`: Cloudflare Worker bindings and fetch entry point.
- `worker-build 0.8.5`: deployable Worker bundle generation in CI.
- Wrangler 4.112.0: local runtime and deployment.

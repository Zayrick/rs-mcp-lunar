# Contract notes

The server preserves the established public tool contract:

- MCP endpoint: exact path `/lunar`.
- Server identity: `Lunar Calendar MCP` version `1.0.0`.
- Ten tools in a stable order, with the same input schemas and Markdown-only results.
- Successful tool calls omit `isError`; invalid inputs and domain failures return text tool errors.
- Other paths return the plain-text server information page.

The native binary and Cloudflare Worker share all tool schemas, validation, domain dispatch, rendered Markdown, identity metadata, and stable tool order. `tests/reference_contract.rs` protects the common contract; Worker-target compilation and the generated Worker bundle are also checked in CI.

| Behavior | Native server | Cloudflare Worker |
| --- | --- | --- |
| Runtime | Axum + rmcp + Tokio | workers-rs + WebAssembly |
| MCP URL | exact `/lunar` | exact `/lunar` |
| Sessions | stateless; no session ID | stateless; no session ID |
| Response transport | rmcp Streamable HTTP | one SSE event per JSON-RPC response |
| Notifications/client responses | handled without a response | `202 Accepted`, empty body |
| Browser Origin policy | wildcard CORS; restrict at proxy | exact `MCP_ALLOWED_ORIGINS` list |
| Request body cap | configure at proxy | 1 MiB in the Worker adapter |

The Worker accepts protocol versions `2024-11-05`, `2025-03-26`, `2025-06-18`, and `2025-11-25`, advertises `2025-11-25` as the latest version, and supports `initialize`, `ping`, `tools/list`, and `tools/call`. MCP JSON-RPC batches are rejected, as required by current MCP lifecycle rules.

`tests/reference_contract.rs` covers the route, tool surface, schemas, errors, and representative output from all ten tools. Domain unit tests cover calendar edges and upstream-library panic boundaries.

## Deliberate corrections

Localized Zi Wei palace lookup uses typed palace identity instead of searching translated output with a Chinese label. This keeps `ziwei_scope_detail` and `ziwei_topic_context` working in all six supported languages.

Unsafe upstream calendar edges are rejected as normal tool errors. BaZi birth years are limited to `0002..=9988`; Zi Wei and period-specific limits are enforced by the relevant domain adapter before invoking upstream convenience APIs.

Native HTTP transport and protocol-version negotiation follow the installed `rmcp` SDK. The Worker adapter follows the same stateless lifecycle and Streamable HTTP request/response shapes without creating server sessions or long-lived GET streams.

# Contract notes

The Cloudflare Worker preserves the established public tool contract:

- Exact MCP endpoint `/lunar`.
- Server identity `Lunar Calendar MCP` version `1.0.0`.
- Ten tools in stable order, with stable input schemas and Markdown-only results.
- Successful calls omit `isError`; invalid inputs and domain failures return text tool errors.
- Other paths return the plain-text server information page.
- Stateless operation with no session ID or long-lived GET stream.

The Worker accepts protocol versions `2024-11-05`, `2025-03-26`, `2025-06-18`, and `2025-11-25`, advertises `2025-11-25` as the latest version, and supports `initialize`, `ping`, `tools/list`, and `tools/call`. Notifications and client responses return `202 Accepted`; MCP JSON-RPC batches are rejected.

`tests/reference_contract.rs` protects the tool surface, schemas, errors, and representative output. Protocol unit tests cover lifecycle and JSON-RPC behavior. CI additionally compiles the Wasm target and performs a Wrangler dry run against the generated bundle.

## Deliberate corrections

Localized Zi Wei palace lookup uses typed palace identity instead of searching translated output with a Chinese label. Unsafe upstream calendar edges are rejected as normal tool errors: BaZi birth years are limited to `0002..=9988`, and the relevant domain adapter enforces Zi Wei and period-specific limits before invoking upstream APIs.

# Contract notes

The server preserves the established public tool contract:

- MCP endpoint: exact path `/lunar`.
- Server identity: `Lunar Calendar MCP` version `1.0.0`.
- Ten tools in a stable order, with the same input schemas and Markdown-only results.
- Successful tool calls omit `isError`; invalid inputs and domain failures return text tool errors.
- Other paths return the plain-text server information page.

`tests/reference_contract.rs` covers the route, tool surface, schemas, errors, and representative output from all ten tools. Domain unit tests cover calendar edges and upstream-library panic boundaries.

## Deliberate corrections

Localized Zi Wei palace lookup uses typed palace identity instead of searching translated output with a Chinese label. This keeps `ziwei_scope_detail` and `ziwei_topic_context` working in all six supported languages.

Unsafe upstream calendar edges are rejected as normal tool errors. BaZi birth years are limited to `0002..=9988`; Zi Wei and period-specific limits are enforced by the relevant domain adapter before invoking upstream convenience APIs.

HTTP transport and protocol-version negotiation follow the installed `rmcp` SDK. The application is stateless and does not create server sessions.

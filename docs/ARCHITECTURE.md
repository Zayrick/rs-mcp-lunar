# Architecture

The project has one runtime target: a native server binary.

```text
HTTP request
    │
    ▼
Axum + rmcp transport
    │
    ▼
contract registry + JSON Schema validation
    │
    ▼
domain dispatcher
   ├── bazi: tyme4rs adapter → typed facts → Markdown
   └── ziwei: iztro adapter → typed facts → Markdown
```

## Responsibilities

- `src/contract.rs` is the single source of truth for server identity, route, tool order, descriptions, and input schemas.
- `src/validation.rs` validates public arguments with `jsonschema`.
- `src/domain/bazi` and `src/domain/ziwei` own calendar adaptation and rendering. Calendar arithmetic stays in the upstream libraries.
- `src/mcp` converts the neutral contract to `rmcp` protocol types.
- `src/transport` owns HTTP routing and SSE transport.
- `src/main.rs` only configures logging, binds the listener, and handles shutdown.

Adding a tool requires one contract entry, one domain use case, one dispatch arm, and tests. Transport code does not calculate calendar facts, and domain code does not build HTTP responses.

## Dependency choices

- `tyme4rs 1.5.0` handles BaZi and sexagenary-calendar arithmetic.
- `iztro 0.9.0` handles Zi Wei natal and horoscope facts.
- `jsonschema 0.48.0` handles schema validation.
- `rmcp 2.2.0` provides the Streamable HTTP MCP server.

XALEN is not included because its current Chinese API does not replace the complete fortune-cycle, localized presentation, and multi-level horoscope data required by these ten tools. Adding a second calculation engine would increase maintenance without removing either existing engine.

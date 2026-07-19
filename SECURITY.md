# Security

- Neither runtime has built-in authentication. An `Authorization` request header is transport-compatible but is not verified by this application.
- The native server binds to loopback by default. Setting `LUNAR_MCP_ADDR` to a public interface is a deployment decision; use a reverse proxy for TLS, authentication, Host/Origin validation, rate limiting, and request-body limits.
- Native wildcard CORS is retained for compatibility and must be restricted at the reverse proxy when the service is not public.
- A deployed Cloudflare Worker is public unless protected with Cloudflare Access or an equivalent gateway policy. Configure WAF/rate-limiting rules appropriate to expected traffic before publishing a stable URL.
- The Worker rejects request bodies larger than 1 MiB. Browser requests with an `Origin` header are rejected unless the exact origin is in the comma-separated `MCP_ALLOWED_ORIGINS` Worker variable; `Origin: null` and wildcard entries are rejected. Non-browser MCP clients may omit `Origin`.
- Keep `MCP_ALLOWED_ORIGINS` as a normal Worker variable because it is not secret. Put production credentials in Cloudflare secrets and local-only values in the ignored `.dev.vars`; never commit them to `wrangler.jsonc`, source, or logs.
- Calendar calls are deterministic and require no runtime secrets or outbound network access.
- The ignored `.reference` directory contains a previously exposed credential. Rotate or revoke it if it belongs to you.
- The repository does not declare a project license. Choose one before redistribution.

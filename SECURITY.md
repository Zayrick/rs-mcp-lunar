# Security

- A deployed Worker is public unless protected by Cloudflare Access or an equivalent gateway. Configure WAF and rate limiting for expected traffic.
- The Worker rejects request bodies larger than 1 MiB.
- Browser requests with an `Origin` header are rejected unless the exact origin appears in the comma-separated `MCP_ALLOWED_ORIGINS` Worker variable. `Origin: null` and wildcard entries are rejected. Non-browser MCP clients may omit `Origin`.
- CORS is not authentication. An `Authorization` header is transport-compatible but is not verified by this application.
- `MCP_ALLOWED_ORIGINS` is a normal variable. Store credentials as Cloudflare secrets and local-only values in ignored `.dev.vars` files; never commit them to source, Wrangler configuration, or logs.
- GitHub deployment credentials belong in the `production` environment as `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`. Scope the token to the target account and Worker deployment permissions.
- Calendar calls are deterministic and require no runtime secrets or outbound network access.
- The ignored `.reference` directory contains a previously exposed credential. Rotate or revoke it if it belongs to you.
- The repository does not declare a project license. Choose one before redistribution.

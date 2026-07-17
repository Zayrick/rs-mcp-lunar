# Security

- The server has no built-in authentication and binds to loopback by default.
- Setting `LUNAR_MCP_ADDR` to a public interface is a deployment decision. Use a reverse proxy for TLS, authentication, Host/Origin validation, rate limiting, and request-body limits.
- Wildcard CORS is retained as part of the MCP contract; restrict it at the reverse proxy when the service is not intended to be public.
- Calendar calls are deterministic and require no runtime secrets or outbound network access.
- The ignored `.reference` directory contains a previously exposed credential. Rotate or revoke it if it belongs to you.
- The repository does not declare a project license. Choose one before redistribution.

//! Cloudflare Workers Streamable HTTP adapter.
//!
//! This deliberately does not reuse `rmcp`'s Tower service: that transport
//! starts Tokio tasks even in stateless mode, while Workers drive Rust futures
//! through the JavaScript event loop and do not provide a Tokio runtime.

use futures_util::StreamExt;
use serde_json::Value;
use worker::{Env, Headers, Method, Request, Response, Result};

use crate::{
    contract::{MCP_PATH, server_info_text},
    mcp::protocol,
};

/// MCP payloads are small JSON messages. Cap buffering so an untrusted
/// chunked request cannot consume the isolate's memory limit.
const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const ALLOWED_ORIGINS_VAR: &str = "MCP_ALLOWED_ORIGINS";

pub async fn handle(request: Request, env: &Env) -> Result<Response> {
    if request.path() != MCP_PATH {
        return info_response();
    }

    let request_origin = request.headers().get("origin")?;
    let allowed_origin = match validate_origin(request_origin.as_deref(), env) {
        Some(origin) => origin,
        None if request_origin.is_some() => {
            return plain_error("Forbidden origin", 403, None, true);
        }
        None => None,
    };

    match request.method() {
        Method::Options => preflight_response(allowed_origin),
        Method::Post => post(request, allowed_origin).await,
        _ => method_not_allowed(allowed_origin),
    }
}

async fn post(mut request: Request, allowed_origin: Option<&str>) -> Result<Response> {
    let content_type = request.headers().get("content-type")?;
    if !content_type
        .as_deref()
        .is_some_and(|value| has_media_type(value, "application/json"))
    {
        return plain_error(
            "Content-Type must be application/json",
            415,
            allowed_origin,
            true,
        );
    }

    let accept = request.headers().get("accept")?;
    let accepts_streamable_http = accept.as_deref().is_some_and(|value| {
        has_media_type(value, "application/json") && has_media_type(value, "text/event-stream")
    });
    if !accepts_streamable_http {
        return plain_error(
            "Accept must include application/json and text/event-stream",
            406,
            allowed_origin,
            true,
        );
    }

    let protocol_header = request.headers().get("mcp-protocol-version")?;
    if protocol_header
        .as_deref()
        .is_some_and(|version| !protocol::is_supported_protocol_version(version))
    {
        return plain_error(
            "Unsupported MCP-Protocol-Version",
            400,
            allowed_origin,
            true,
        );
    }

    let mut bytes = Vec::new();
    let mut stream = match request.stream() {
        Ok(stream) => stream,
        Err(_) => {
            return sse_response(protocol::parse_error(), 400, allowed_origin);
        }
    };
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if chunk.len() > MAX_REQUEST_BYTES.saturating_sub(bytes.len()) {
            return plain_error("Request body exceeds 1 MiB", 413, allowed_origin, true);
        }
        bytes.extend_from_slice(&chunk);
    }

    let message: Value = match serde_json::from_slice(&bytes) {
        Ok(message) => message,
        Err(_) => return sse_response(protocol::parse_error(), 400, allowed_origin),
    };

    if let (Some(header), Some(requested)) = (
        protocol_header.as_deref(),
        protocol::initialize_protocol_version(&message),
    ) && header != requested
    {
        return plain_error(
            "MCP-Protocol-Version does not match initialize params",
            400,
            allowed_origin,
            true,
        );
    }

    match protocol::handle(message) {
        Some(response) => sse_response(response, 200, allowed_origin),
        None => accepted_response(allowed_origin),
    }
}

fn validate_origin<'a>(request_origin: Option<&'a str>, env: &Env) -> Option<Option<&'a str>> {
    let Some(origin) = request_origin else {
        return Some(None);
    };
    if origin == "null" {
        return None;
    }

    let configured = env.var(ALLOWED_ORIGINS_VAR).ok()?;
    configured
        .to_string()
        .split(',')
        .map(str::trim)
        .any(|candidate| !candidate.is_empty() && candidate != "*" && candidate == origin)
        .then_some(Some(origin))
}

fn has_media_type(header: &str, expected: &str) -> bool {
    header.split(',').any(|entry| {
        entry
            .split(';')
            .next()
            .is_some_and(|media_type| media_type.trim().eq_ignore_ascii_case(expected))
    })
}

fn info_response() -> Result<Response> {
    let mut response = Response::ok(server_info_text())?;
    response
        .headers_mut()
        .set("X-Content-Type-Options", "nosniff")?;
    Ok(response)
}

fn preflight_response(allowed_origin: Option<&str>) -> Result<Response> {
    let response = Response::empty()?.with_status(204);
    with_mcp_headers(response, allowed_origin, false)
}

fn accepted_response(allowed_origin: Option<&str>) -> Result<Response> {
    let response = Response::empty()?.with_status(202);
    with_mcp_headers(response, allowed_origin, true)
}

fn method_not_allowed(allowed_origin: Option<&str>) -> Result<Response> {
    let mut response = Response::error("Method Not Allowed", 405)?;
    response.headers_mut().set("Allow", "POST, OPTIONS")?;
    with_mcp_headers(response, allowed_origin, true)
}

fn plain_error(
    message: &str,
    status: u16,
    allowed_origin: Option<&str>,
    mcp_headers: bool,
) -> Result<Response> {
    let response = Response::error(message, status)?;
    if mcp_headers {
        with_mcp_headers(response, allowed_origin, true)
    } else {
        Ok(response)
    }
}

fn sse_response(message: Value, status: u16, allowed_origin: Option<&str>) -> Result<Response> {
    let payload = serde_json::to_string(&message)?;
    let mut response = Response::ok(format!("data: {payload}\n\n"))?.with_status(status);
    response
        .headers_mut()
        .set("Content-Type", "text/event-stream; charset=utf-8")?;
    response.headers_mut().set("Cache-Control", "no-cache")?;
    with_mcp_headers(response, allowed_origin, false)
}

fn with_mcp_headers(
    mut response: Response,
    allowed_origin: Option<&str>,
    no_store: bool,
) -> Result<Response> {
    let headers: &mut Headers = response.headers_mut();
    headers.set("Access-Control-Allow-Methods", "POST, OPTIONS")?;
    headers.set(
        "Access-Control-Allow-Headers",
        "Content-Type, Accept, Authorization, MCP-Protocol-Version, MCP-Session-Id, Last-Event-ID",
    )?;
    headers.set("Access-Control-Expose-Headers", "MCP-Session-Id")?;
    headers.set("X-Content-Type-Options", "nosniff")?;
    if no_store {
        headers.set("Cache-Control", "no-store")?;
    }
    if let Some(origin) = allowed_origin {
        headers.set("Access-Control-Allow-Origin", origin)?;
        headers.set("Vary", "Origin, Accept")?;
    } else {
        headers.set("Vary", "Accept")?;
    }
    Ok(response)
}

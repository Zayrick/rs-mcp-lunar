use axum::{
    Router,
    http::{HeaderName, HeaderValue, header},
    response::IntoResponse,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
};

use crate::mcp::{LunarMcpServer, MCP_PATH, server_info_text};

/// Build the stateless HTTP application.
///
/// `route_service` is intentionally used instead of `nest_service`: only the
/// exact `/lunar` pathname is MCP, matching the public contract.
pub fn app() -> Router {
    let config = StreamableHttpServerConfig::default()
        .with_stateful_mode(false)
        .with_sse_keep_alive(None)
        // Preserve the existing public Host behavior. Internet-facing
        // deployments should validate Host and Origin at the reverse proxy.
        .disable_allowed_hosts();
    let mcp: StreamableHttpService<LunarMcpServer, LocalSessionManager> =
        StreamableHttpService::new(|| Ok(LunarMcpServer), Default::default(), config);
    let mcp = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::if_not_present(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::ACCESS_CONTROL_EXPOSE_HEADERS,
            HeaderValue::from_static("mcp-session-id"),
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .expose_headers([HeaderName::from_static("mcp-session-id")]),
        )
        .service(mcp);

    Router::new()
        .route_service(MCP_PATH, mcp)
        .fallback(server_info)
}

async fn server_info() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        server_info_text(),
    )
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn non_mcp_paths_return_info() {
        for path in ["/", "/lunar/", "/lunar/subpath", "/anything"] {
            let response = app()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{path}");
            assert_eq!(
                response.headers()[header::CONTENT_TYPE],
                "text/plain; charset=utf-8"
            );
            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert!(String::from_utf8_lossy(&body).contains("Lunar Calendar MCP Server"));
        }
    }

    #[tokio::test]
    async fn exact_mcp_path_is_sse() {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
        let response = app()
            .oneshot(
                Request::post("/lunar?query=preserved")
                    .header(header::HOST, "example.com")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::ACCEPT, "application/json, text/event-stream")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .contains("text/event-stream")
        );
        assert_eq!(response.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
        assert_eq!(
            response.headers()[header::ACCESS_CONTROL_EXPOSE_HEADERS],
            "mcp-session-id"
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(String::from_utf8_lossy(&body).contains("bazi_chart"));
    }
}

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use rs_mcp_lunar::app;
use serde_json::{Value, json};
use tower::ServiceExt;

const TOOL_NAMES: [&str; 10] = [
    "bazi_chart",
    "bazi_structure",
    "bazi_timeline",
    "bazi_period_detail",
    "bazi_shensha",
    "ziwei_chart",
    "ziwei_palace_detail",
    "ziwei_horoscope_overview",
    "ziwei_scope_detail",
    "ziwei_topic_context",
];

const LEGACY_TOOL_NAMES: [&str; 11] = [
    "convert_to_ganzhi",
    "get_current_ganzhi",
    "get_bazi_chart",
    "get_bazi_shensha",
    "get_bazi_fortune",
    "get_bazi_flow_month",
    "get_bazi_flow_day",
    "get_bazi_flow_hour",
    "get_ziwei_chart",
    "get_ziwei_horoscope",
    "get_ziwei_scope_detail",
];

async fn request(path: &str, method: &str, params: Value) -> Value {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });
    let response = app()
        .oneshot(
            Request::post(path)
                .header(header::HOST, "example.com")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCEPT, "application/json, text/event-stream")
                .body(Body::from(payload.to_string()))
                .expect("valid request"),
        )
        .await
        .expect("MCP response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("read response")
        .to_bytes();
    let body = String::from_utf8(body.to_vec()).expect("UTF-8 SSE");
    let data = body
        .lines()
        .find_map(|line| line.strip_prefix("data: "))
        .expect("SSE data line");
    serde_json::from_str(data).expect("JSON-RPC data")
}

async fn call_tool(name: &str, arguments: Value) -> Value {
    request(
        "/lunar",
        "tools/call",
        json!({"name": name, "arguments": arguments}),
    )
    .await["result"]
        .clone()
}

fn markdown(result: &Value) -> &str {
    assert_ne!(result.get("isError"), Some(&Value::Bool(true)));
    assert!(result.get("structuredContent").is_none());
    assert!(result.get("isError").is_none());
    let content = result["content"].as_array().expect("content array");
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "text");
    content[0]["text"].as_str().expect("Markdown text")
}

#[tokio::test]
async fn root_and_subpaths_match_reference_fallback() {
    for path in ["/", "/lunar/", "/lunar/test", "/other"] {
        let response = app()
            .oneshot(
                Request::get(path)
                    .body(Body::empty())
                    .expect("valid request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/plain; charset=utf-8"
        );
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let body = String::from_utf8_lossy(&body);
        assert!(body.contains("Lunar Calendar MCP Server"));
        assert!(body.contains("MCP endpoint: /lunar"));
        for name in TOOL_NAMES {
            assert!(body.contains(&format!("| {name} |")));
        }
        for name in LEGACY_TOOL_NAMES {
            assert!(!body.contains(name));
        }
    }
}

#[tokio::test]
async fn exposes_exact_tool_surface_and_schema_boundaries() {
    let response = request("/lunar?query=kept", "tools/list", json!({})).await;
    let tools = response["result"]["tools"].as_array().expect("tools array");
    let names = tools
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name"))
        .collect::<Vec<_>>();
    assert_eq!(names, TOOL_NAMES);
    for tool in tools {
        let description = tool["description"].as_str().expect("description");
        assert!(description.contains("适用场景"));
        assert!(description.contains("不要"));
        assert!(description.contains("下一步"));
        assert!(tool.get("outputSchema").is_none());
        assert_eq!(tool["execution"]["taskSupport"], "forbidden");
    }
}

#[tokio::test]
async fn all_ten_tools_return_markdown_only() {
    let cases = [
        (
            "bazi_chart",
            json!({"datetime":"2024-01-15 08:30","gender":"男"}),
            "八字本命基础盘",
        ),
        (
            "bazi_structure",
            json!({"datetime":"2024-01-15 08:30","gender":"男"}),
            "八字命局结构证据",
        ),
        (
            "bazi_timeline",
            json!({"datetime":"2024-01-15 08:30","gender":"男","startYear":2026,"count":2}),
            "八字大运流年时间轴",
        ),
        (
            "bazi_period_detail",
            json!({"datetime":"2024-01-15 08:30","gender":"男","scope":"day","date":"2026-06-12"}),
            "八字单一周期详盘",
        ),
        (
            "bazi_shensha",
            json!({"datetime":"2024-01-15 08:30"}),
            "常用八字神煞辅助表",
        ),
        (
            "ziwei_chart",
            json!({"datetime":"2024-01-15 08:30","gender":"男","profile":"sanhe"}),
            "紫微斗数本命全盘",
        ),
        (
            "ziwei_palace_detail",
            json!({"datetime":"2024-01-15 08:30","gender":"男","palace":"命宫"}),
            "紫微斗数单宫详盘",
        ),
        (
            "ziwei_horoscope_overview",
            json!({"birthDatetime":"2024-01-15 08:30","gender":"男","targetDatetime":"2026-06-12 18:00","profile":"sanhe"}),
            "紫微运限总览",
        ),
        (
            "ziwei_scope_detail",
            json!({"birthDatetime":"2024-01-15 08:30","gender":"男","targetDatetime":"2026-06-12 18:00","scope":"yearly","focusPalace":"命宫"}),
            "紫微单层运限详盘",
        ),
        (
            "ziwei_topic_context",
            json!({"birthDatetime":"2024-01-15 08:30","gender":"男","targetDatetime":"2026-06-12 18:00","topic":"career"}),
            "紫微专题取证",
        ),
    ];

    for (name, arguments, title) in cases {
        let result = call_tool(name, arguments).await;
        let text = markdown(&result);
        assert!(text.contains(title), "{name}");
        assert!(text.lines().any(|line| line.starts_with("| ")), "{name}");
    }
}

#[tokio::test]
async fn ganzhi_periods_include_reference_gregorian_ranges() {
    let timeline = call_tool(
        "bazi_timeline",
        json!({"datetime":"2024-01-15 08:30","gender":"男","startYear":2026,"count":1}),
    )
    .await;
    assert!(markdown(&timeline).contains("| 年份 | 公历实际对应范围 | 年龄 |"));

    let month = call_tool(
        "bazi_period_detail",
        json!({"datetime":"2024-01-15 08:30","gender":"男","scope":"month","year":2026,"month":5}),
    )
    .await;
    assert!(markdown(&month).contains("公历实际对应范围: 2026-06-05"));

    let hour = call_tool(
        "bazi_period_detail",
        json!({"datetime":"2024-01-15 08:30","gender":"男","scope":"hour","date":"2026-06-12","hour":18}),
    )
    .await;
    assert!(
        markdown(&hour).contains("公历实际对应范围: 2026-06-12 17:00:00 至 2026-06-12 18:59:59")
    );

    let overview = call_tool(
        "ziwei_horoscope_overview",
        json!({"birthDatetime":"2024-01-15 08:30","gender":"男","targetDatetime":"2026-06-12 18:00"}),
    )
    .await;
    let overview = markdown(&overview);
    assert!(overview.contains("| 层级 | 干支 | 公历实际对应范围 |"));
    assert!(overview.contains("2026-06-12 17:00:00 至 2026-06-12 18:59:59"));
}

#[tokio::test]
async fn tool_responsibility_boundaries_are_preserved() {
    let structure = call_tool(
        "bazi_structure",
        json!({"datetime":"2024-01-15 08:30","gender":"男"}),
    )
    .await;
    let structure = markdown(&structure);
    assert!(structure.contains("不输出最终断命结论"));
    assert!(!structure.contains("公历实际对应范围"));

    let overview = call_tool(
        "ziwei_horoscope_overview",
        json!({"birthDatetime":"2024-01-15 08:30","gender":"男","targetDatetime":"2026-06-12 18:00"}),
    )
    .await;
    let overview = markdown(&overview);
    assert!(overview.contains("只做导航"));
    assert!(overview.contains("下一步必须调用 ziwei_scope_detail"));

    let topic = call_tool(
        "ziwei_topic_context",
        json!({"birthDatetime":"2024-01-15 08:30","gender":"男","targetDatetime":"2026-06-12 18:00","topic":"career"}),
    )
    .await;
    let topic = markdown(&topic);
    assert!(topic.len() < 6_000);
    for palace in ["官禄", "迁移", "财帛", "福德"] {
        assert!(topic.contains(&format!("| {palace} |")));
    }
    assert!(topic.contains("ziwei_palace_detail(palace=官禄)"));
    assert!(topic.contains("ziwei_scope_detail(scope=yearly, focusPalace=官禄)"));
}

#[tokio::test]
async fn invalid_arguments_are_text_tool_errors() {
    let cases = [
        ("bazi_chart", json!({"datetime":"not-a-date","gender":"男"})),
        (
            "bazi_chart",
            json!({"datetime":"2024-01-15 08:30","gender":"unknown"}),
        ),
        (
            "bazi_period_detail",
            json!({"datetime":"2024-01-15 08:30","gender":"男","scope":"week","date":"2026-06-12"}),
        ),
        (
            "ziwei_topic_context",
            json!({"birthDatetime":"2024-01-15 08:30","gender":"男","targetDatetime":"2026-06-12 18:00","topic":"education"}),
        ),
        (
            "ziwei_chart",
            json!({"datetime":"2024-01-15 08:30","gender":"男","profile":"legacy"}),
        ),
        (
            "ziwei_chart",
            json!({"datetime":"2024-01-15 08:30","gender":"男","calendar":"gregorian"}),
        ),
    ];

    for (name, arguments) in cases {
        let result = call_tool(name, arguments).await;
        assert_eq!(result["isError"], true, "{name}");
        assert!(
            !result["content"][0]["text"]
                .as_str()
                .expect("error text")
                .is_empty()
        );
    }
}

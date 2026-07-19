use rs_mcp_lunar::{contract::server_info_text, mcp::protocol};
use serde_json::{Value, json};

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

fn request(method: &str, params: Value) -> Value {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });
    protocol::handle(payload).expect("JSON-RPC response")
}

fn call_tool(name: &str, arguments: Value) -> Value {
    request("tools/call", json!({"name": name, "arguments": arguments}))["result"].clone()
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

#[test]
fn server_info_matches_reference_fallback() {
    let body = server_info_text();
    assert!(body.contains("Lunar Calendar MCP Server"));
    assert!(body.contains("MCP endpoint: /lunar"));
    for name in TOOL_NAMES {
        assert!(body.contains(&format!("| {name} |")));
    }
    for name in LEGACY_TOOL_NAMES {
        assert!(!body.contains(name));
    }
}

#[test]
fn exposes_exact_tool_surface_and_schema_boundaries() {
    let response = request("tools/list", json!({}));
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

#[test]
fn all_ten_tools_return_markdown_only() {
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
        let result = call_tool(name, arguments);
        let text = markdown(&result);
        assert!(text.contains(title), "{name}");
        assert!(text.lines().any(|line| line.starts_with("| ")), "{name}");
    }
}

#[test]
fn ganzhi_periods_include_reference_gregorian_ranges() {
    let timeline = call_tool(
        "bazi_timeline",
        json!({"datetime":"2024-01-15 08:30","gender":"男","startYear":2026,"count":1}),
    );
    assert!(markdown(&timeline).contains("| 年份 | 公历实际对应范围 | 年龄 |"));

    let month = call_tool(
        "bazi_period_detail",
        json!({"datetime":"2024-01-15 08:30","gender":"男","scope":"month","year":2026,"month":5}),
    );
    assert!(markdown(&month).contains("公历实际对应范围: 2026-06-05"));

    let hour = call_tool(
        "bazi_period_detail",
        json!({"datetime":"2024-01-15 08:30","gender":"男","scope":"hour","date":"2026-06-12","hour":18}),
    );
    assert!(
        markdown(&hour).contains("公历实际对应范围: 2026-06-12 17:00:00 至 2026-06-12 18:59:59")
    );

    let overview = call_tool(
        "ziwei_horoscope_overview",
        json!({"birthDatetime":"2024-01-15 08:30","gender":"男","targetDatetime":"2026-06-12 18:00"}),
    );
    let overview = markdown(&overview);
    assert!(overview.contains("| 层级 | 干支 | 公历实际对应范围 |"));
    assert!(overview.contains("2026-06-12 17:00:00 至 2026-06-12 18:59:59"));
}

#[test]
fn tool_responsibility_boundaries_are_preserved() {
    let structure = call_tool(
        "bazi_structure",
        json!({"datetime":"2024-01-15 08:30","gender":"男"}),
    );
    let structure = markdown(&structure);
    assert!(structure.contains("不输出最终断命结论"));
    assert!(!structure.contains("公历实际对应范围"));

    let overview = call_tool(
        "ziwei_horoscope_overview",
        json!({"birthDatetime":"2024-01-15 08:30","gender":"男","targetDatetime":"2026-06-12 18:00"}),
    );
    let overview = markdown(&overview);
    assert!(overview.contains("只做导航"));
    assert!(overview.contains("下一步必须调用 ziwei_scope_detail"));

    let topic = call_tool(
        "ziwei_topic_context",
        json!({"birthDatetime":"2024-01-15 08:30","gender":"男","targetDatetime":"2026-06-12 18:00","topic":"career"}),
    );
    let topic = markdown(&topic);
    assert!(topic.len() < 6_000);
    for palace in ["官禄", "迁移", "财帛", "福德"] {
        assert!(topic.contains(&format!("| {palace} |")));
    }
    assert!(topic.contains("ziwei_palace_detail(palace=官禄)"));
    assert!(topic.contains("ziwei_scope_detail(scope=yearly, focusPalace=官禄)"));
}

#[test]
fn invalid_arguments_are_text_tool_errors() {
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
        let result = call_tool(name, arguments);
        assert_eq!(result["isError"], true, "{name}");
        assert!(
            !result["content"][0]["text"]
                .as_str()
                .expect("error text")
                .is_empty()
        );
    }
}

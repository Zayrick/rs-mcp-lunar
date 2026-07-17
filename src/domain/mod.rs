//! Domain entry points. Library-specific types stay behind these modules.

pub mod bazi;
pub mod ziwei;

use serde_json::{Map, Value};

/// Dispatch a public MCP tool to its domain engine.
pub fn execute(tool: &str, arguments: &Map<String, Value>) -> Result<String, String> {
    match tool {
        "bazi_chart" | "bazi_structure" | "bazi_timeline" | "bazi_period_detail"
        | "bazi_shensha" => bazi::execute(tool, arguments),
        "ziwei_chart"
        | "ziwei_palace_detail"
        | "ziwei_horoscope_overview"
        | "ziwei_scope_detail"
        | "ziwei_topic_context" => ziwei::execute(tool, arguments),
        _ => Err(format!("Unknown tool: {tool}")),
    }
}

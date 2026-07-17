use std::sync::Arc;

use rmcp::model::{TaskSupport, Tool, ToolExecution};

pub use crate::contract::{MCP_PATH, SERVER_NAME, SERVER_VERSION, server_info_text};

pub fn tools() -> Vec<Tool> {
    crate::contract::tool_specs().iter().map(to_tool).collect()
}

pub fn tool(name: &str) -> Option<Tool> {
    crate::contract::find_tool(name).map(to_tool)
}

fn to_tool(spec: &crate::contract::ToolSpec) -> Tool {
    let mut tool = Tool::new(spec.name, spec.description, Arc::new(spec.input_schema()))
        .with_title(spec.title);
    tool.execution = Some(ToolExecution::new().with_task_support(TaskSupport::Forbidden));
    tool
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_preserves_reference_order_and_shape() {
        let tools = tools();
        let names = tools
            .iter()
            .map(|tool| tool.name.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
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
            ]
        );
        assert!(tools.iter().all(|tool| tool.output_schema.is_none()));
        assert!(tools.iter().all(|tool| {
            tool.execution
                .as_ref()
                .and_then(|execution| execution.task_support)
                == Some(TaskSupport::Forbidden)
        }));
    }
}

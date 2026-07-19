use rmcp::{
    ErrorData as McpError, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, ContentBlock, Implementation, ListToolsResult,
        PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
    },
    service::{RequestContext, RoleServer},
};

use super::protocol::{ToolCallOutcome, call_tool};
use super::registry;
use crate::contract::{SERVER_NAME, SERVER_VERSION};

#[derive(Debug, Clone, Copy, Default)]
pub struct LunarMcpServer;

impl ServerHandler for LunarMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .build(),
        )
        .with_server_info(Implementation::new(SERVER_NAME, SERVER_VERSION))
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult::with_all_items(registry::tools())))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        registry::tool(name)
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        let name = request.name.into_owned();
        let result = match call_tool(&name, request.arguments.as_ref()) {
            ToolCallOutcome::Success(markdown) => {
                let mut result = CallToolResult::success(vec![ContentBlock::text(markdown)]);
                // The reference SDK omits `isError` on successful results.
                result.is_error = None;
                result
            }
            ToolCallOutcome::Error(message) => {
                CallToolResult::error(vec![ContentBlock::text(message)])
            }
        };
        std::future::ready(Ok(result))
    }
}

//! Runtime-neutral, stateless MCP/JSON-RPC dispatcher.
//!
//! The Cloudflare adapter maps these values into Streamable HTTP events.
//! Keeping protocol dispatch separate from Worker HTTP types makes the public
//! contract directly testable on the host toolchain.

use serde_json::{Map, Value, json};

use crate::contract::{SERVER_NAME, SERVER_VERSION};

/// Latest stable protocol version advertised by this Worker.
pub const LATEST_PROTOCOL_VERSION: &str = "2025-11-25";

/// Protocol versions understood by both runtime adapters.
pub const SUPPORTED_PROTOCOL_VERSIONS: [&str; 4] =
    ["2024-11-05", "2025-03-26", "2025-06-18", "2025-11-25"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ToolCallOutcome {
    Success(String),
    Error(String),
}

/// Execute one public tool without coupling the result to an MCP SDK type.
pub(crate) fn call_tool(name: &str, arguments: Option<&Map<String, Value>>) -> ToolCallOutcome {
    if crate::contract::find_tool(name).is_none() {
        return ToolCallOutcome::Error(format!("MCP error -32602: Tool {name} not found"));
    }
    let Some(arguments) = arguments else {
        return ToolCallOutcome::Error(crate::validation::missing_arguments(name));
    };
    if let Err(message) = crate::validation::arguments(name, arguments) {
        return ToolCallOutcome::Error(message);
    }

    match crate::domain::execute(name, arguments) {
        Ok(markdown) => ToolCallOutcome::Success(markdown),
        Err(message) => ToolCallOutcome::Error(format!("{name} 错误: {message}")),
    }
}

/// Dispatch one MCP JSON-RPC message.
///
/// MCP removed JSON-RPC batching, so arrays are rejected as invalid requests.
/// Valid notifications and client responses intentionally return `None`; the
/// HTTP adapter translates that into `202 Accepted`.
pub fn handle(message: Value) -> Option<Value> {
    let Value::Object(object) = message else {
        return Some(error(Value::Null, -32600, "Invalid Request", None));
    };

    let id = match valid_id(&object) {
        Ok(id) => id,
        Err(()) => {
            return Some(error(Value::Null, -32600, "Invalid Request", None));
        }
    };
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Some(error(
            id.unwrap_or(Value::Null),
            -32600,
            "Invalid Request",
            None,
        ));
    }

    // A Streamable HTTP POST may carry a response to a server request. This
    // stateless server never emits requests, but accepting the shape is still
    // transport-compliant and avoids replying to a response.
    if object.get("method").is_none()
        && object.contains_key("id")
        && (object.contains_key("result") || object.contains_key("error"))
    {
        return None;
    }

    let Some(method) = object.get("method").and_then(Value::as_str) else {
        return Some(error(
            id.unwrap_or(Value::Null),
            -32600,
            "Invalid Request",
            None,
        ));
    };

    // JSON-RPC notifications never receive a response. The server has no
    // notification-side effects, so there is no background work to schedule.
    let id = id?;

    match dispatch(method, object.get("params")) {
        Ok(result) => Some(json!({"jsonrpc": "2.0", "id": id, "result": result})),
        Err(failure) => Some(error(id, failure.code, failure.message, failure.data)),
    }
}

/// JSON-RPC parse error used when the HTTP body is not valid JSON.
pub fn parse_error() -> Value {
    error(Value::Null, -32700, "Parse error", None)
}

pub fn is_supported_protocol_version(version: &str) -> bool {
    SUPPORTED_PROTOCOL_VERSIONS.contains(&version)
}

/// Return the version requested by an `initialize` message, if present.
pub fn initialize_protocol_version(message: &Value) -> Option<&str> {
    let object = message.as_object()?;
    (object.get("method")?.as_str()? == "initialize")
        .then(|| {
            object
                .get("params")?
                .as_object()?
                .get("protocolVersion")?
                .as_str()
        })
        .flatten()
}

fn dispatch(method: &str, params: Option<&Value>) -> Result<Value, RpcFailure> {
    match method {
        "initialize" => initialize(params),
        "ping" => {
            optional_object(params)?;
            Ok(json!({}))
        }
        "tools/list" => {
            optional_object(params)?;
            Ok(json!({"tools": tools()}))
        }
        "tools/call" => tools_call(params),
        _ => Err(RpcFailure::new(-32601, "Method not found")),
    }
}

fn initialize(params: Option<&Value>) -> Result<Value, RpcFailure> {
    let params = required_object(params, crate::validation::missing_request_params())?;
    let Some(requested) = params.get("protocolVersion").and_then(Value::as_str) else {
        return Err(RpcFailure::invalid_params(
            "params.protocolVersion must be a string",
        ));
    };
    if !params.get("capabilities").is_some_and(Value::is_object) {
        return Err(RpcFailure::invalid_params(
            "params.capabilities must be an object",
        ));
    }
    let Some(client_info) = params.get("clientInfo").and_then(Value::as_object) else {
        return Err(RpcFailure::invalid_params(
            "params.clientInfo must be an object",
        ));
    };
    if !client_info.get("name").is_some_and(Value::is_string) {
        return Err(RpcFailure::invalid_params(
            "params.clientInfo.name must be a string",
        ));
    }
    if !client_info.get("version").is_some_and(Value::is_string) {
        return Err(RpcFailure::invalid_params(
            "params.clientInfo.version must be a string",
        ));
    }
    let negotiated = if is_supported_protocol_version(requested) {
        requested
    } else {
        LATEST_PROTOCOL_VERSION
    };

    Ok(json!({
        "protocolVersion": negotiated,
        "capabilities": {
            "tools": {"listChanged": true}
        },
        "serverInfo": {
            "name": SERVER_NAME,
            "version": SERVER_VERSION
        }
    }))
}

fn tools_call(params: Option<&Value>) -> Result<Value, RpcFailure> {
    let params = required_object(params, crate::validation::missing_tool_call_params())?;
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        let received = params
            .get("name")
            .map(json_type_name)
            .unwrap_or("undefined");
        return Err(RpcFailure::invalid_params(
            crate::validation::invalid_tool_name(received),
        ));
    };

    let arguments = match params.get("arguments") {
        Some(Value::Object(arguments)) => Some(arguments),
        Some(value) => {
            return Err(RpcFailure::invalid_params(
                crate::validation::invalid_arguments_shape(json_type_name(value)),
            ));
        }
        None => None,
    };

    let result = match call_tool(name, arguments) {
        ToolCallOutcome::Success(markdown) => json!({
            "content": [{"type": "text", "text": markdown}]
        }),
        ToolCallOutcome::Error(message) => json!({
            "content": [{"type": "text", "text": message}],
            "isError": true
        }),
    };
    Ok(result)
}

fn tools() -> Vec<Value> {
    crate::contract::tool_specs()
        .iter()
        .map(|spec| {
            json!({
                "name": spec.name,
                "title": spec.title,
                "description": spec.description,
                "inputSchema": spec.input_schema(),
                "execution": {"taskSupport": "forbidden"}
            })
        })
        .collect()
}

fn optional_object(params: Option<&Value>) -> Result<Option<&Map<String, Value>>, RpcFailure> {
    match params {
        None => Ok(None),
        Some(Value::Object(params)) => Ok(Some(params)),
        Some(value) => Err(RpcFailure::invalid_params(format!(
            "params must be an object, received {}",
            json_type_name(value)
        ))),
    }
}

fn required_object(
    params: Option<&Value>,
    missing_message: String,
) -> Result<&Map<String, Value>, RpcFailure> {
    match params {
        Some(Value::Object(params)) => Ok(params),
        Some(value) => Err(RpcFailure::invalid_params(format!(
            "params must be an object, received {}",
            json_type_name(value)
        ))),
        None => Err(RpcFailure::invalid_params(missing_message)),
    }
}

fn valid_id(object: &Map<String, Value>) -> Result<Option<Value>, ()> {
    let Some(id) = object.get("id") else {
        return Ok(None);
    };
    match id {
        Value::Null | Value::String(_) => Ok(Some(id.clone())),
        Value::Number(number) if number.as_i64().is_some() || number.as_u64().is_some() => {
            Ok(Some(id.clone()))
        }
        _ => Err(()),
    }
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn error(id: Value, code: i32, message: &str, data: Option<String>) -> Value {
    let mut body = json!({"code": code, "message": message});
    if let Some(data) = data {
        body["data"] = Value::String(data);
    }
    json!({"jsonrpc": "2.0", "id": id, "error": body})
}

struct RpcFailure {
    code: i32,
    message: &'static str,
    data: Option<String>,
}

impl RpcFailure {
    fn new(code: i32, message: &'static str) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }

    fn invalid_params(data: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: "Invalid params",
            data: Some(data.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: Value, method: &str, params: Value) -> Value {
        json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
    }

    #[test]
    fn initializes_and_negotiates_known_or_unknown_versions() {
        let known = handle(request(
            json!("init-1"),
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1"}
            }),
        ))
        .expect("response");
        assert_eq!(known["id"], "init-1");
        assert_eq!(known["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(known["result"]["serverInfo"]["name"], SERVER_NAME);

        let unknown = handle(request(
            json!(2),
            "initialize",
            json!({
                "protocolVersion": "2099-01-01",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1"}
            }),
        ))
        .expect("response");
        assert_eq!(
            unknown["result"]["protocolVersion"],
            LATEST_PROTOCOL_VERSION
        );

        let malformed = handle(request(
            json!(3),
            "initialize",
            json!({"protocolVersion": "2025-11-25", "capabilities": {}}),
        ))
        .expect("response");
        assert_eq!(malformed["error"]["code"], -32602);
    }

    #[test]
    fn lists_the_exact_tool_contract() {
        let response = handle(request(json!(1), "tools/list", json!({}))).expect("response");
        let tools = response["result"]["tools"].as_array().expect("tools");
        let names = tools
            .iter()
            .map(|tool| tool["name"].as_str().expect("name"))
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
        assert!(tools.iter().all(|tool| tool["inputSchema"].is_object()));
        assert!(
            tools
                .iter()
                .all(|tool| tool["execution"]["taskSupport"] == "forbidden")
        );
    }

    #[test]
    fn calls_tools_with_the_public_result_shape() {
        let response = handle(request(
            json!(1),
            "tools/call",
            json!({
                "name": "bazi_chart",
                "arguments": {"datetime": "2024-01-15 08:30", "gender": "男"}
            }),
        ))
        .expect("response");
        assert!(
            response["result"]["content"][0]["text"]
                .as_str()
                .expect("text")
                .contains("八字本命基础盘")
        );
        assert!(response["result"].get("isError").is_none());
        assert!(response["result"].get("structuredContent").is_none());

        let invalid = handle(request(
            json!(2),
            "tools/call",
            json!({"name": "bazi_chart", "arguments": {}}),
        ))
        .expect("response");
        assert_eq!(invalid["result"]["isError"], true);
    }

    #[test]
    fn notifications_and_client_responses_do_not_get_responses() {
        assert!(
            handle(json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            }))
            .is_none()
        );
        assert!(handle(json!({"jsonrpc": "2.0", "id": 4, "result": {}})).is_none());
    }

    #[test]
    fn rejects_batches_and_unknown_methods_with_json_rpc_errors() {
        let batch = handle(json!([])).expect("error");
        assert_eq!(batch["error"]["code"], -32600);

        let unknown = handle(request(json!(7), "unknown/method", json!({}))).expect("error");
        assert_eq!(unknown["id"], 7);
        assert_eq!(unknown["error"]["code"], -32601);

        for invalid_id in [json!(true), json!([]), json!({}), json!(1.5)] {
            let invalid = handle(request(invalid_id, "ping", json!({}))).expect("error");
            assert_eq!(invalid["id"], Value::Null);
            assert_eq!(invalid["error"]["code"], -32600);
        }
    }
}

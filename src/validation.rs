use std::{collections::HashMap, sync::OnceLock};

use jsonschema::error::{TypeKind, ValidationErrorKind};
use jsonschema::types::JsonType;
use serde::Serialize;
use serde_json::{Map, Value};

static VALIDATORS: OnceLock<Result<HashMap<&'static str, jsonschema::Validator>, String>> =
    OnceLock::new();

/// Match the reference SDK's error when a tool call omits `arguments`.
pub fn missing_arguments(tool: &str) -> String {
    format!(
        "MCP error -32602: Input validation error: Invalid arguments for tool {tool}: [\n  {{\n    \"expected\": \"object\",\n    \"code\": \"invalid_type\",\n    \"path\": [],\n    \"message\": \"Invalid input: expected object, received undefined\"\n  }}\n]"
    )
}

/// Match the reference transport's pre-handler error for a non-object
/// `params.arguments` value.
pub fn invalid_arguments_shape(received: &str) -> String {
    format!(
        "[\n  {{\n    \"expected\": \"record\",\n    \"code\": \"invalid_type\",\n    \"path\": [\n      \"params\",\n      \"arguments\"\n    ],\n    \"message\": \"Invalid input: expected record, received {received}\"\n  }}\n]"
    )
}

/// Match the reference method-schema error when `tools/call` omits `params`.
pub fn missing_tool_call_params() -> String {
    missing_request_params()
}

/// Match the reference method-schema error when a method requiring `params`
/// omits that object.
pub fn missing_request_params() -> String {
    "[\n  {\n    \"expected\": \"object\",\n    \"code\": \"invalid_type\",\n    \"path\": [\n      \"params\"\n    ],\n    \"message\": \"Invalid input: expected object, received undefined\"\n  }\n]"
        .to_owned()
}

/// Match the reference method-schema error for an absent or non-string tool
/// name. `received` uses Zod's JSON type names (or `undefined`).
pub fn invalid_tool_name(received: &str) -> String {
    format!(
        "[\n  {{\n    \"expected\": \"string\",\n    \"code\": \"invalid_type\",\n    \"path\": [\n      \"params\",\n      \"name\"\n    ],\n    \"message\": \"Invalid input: expected string, received {received}\"\n  }}\n]"
    )
}

/// Validate arguments against the exact public tool schema.
///
/// Domain services still enforce cross-field rules such as `scope=hour`
/// requiring both `date` and `hour`; JSON Schema owns shape, enum and range
/// validation so those rules are not duplicated across transports.
pub fn arguments(tool: &str, arguments: &Map<String, Value>) -> Result<(), String> {
    let validators = VALIDATORS
        .get_or_init(build_validators)
        .as_ref()
        .map_err(Clone::clone)?;
    let Some(validator) = validators.get(tool) else {
        return Ok(());
    };
    let instance = Value::Object(arguments.clone());
    let errors = validator.iter_errors(&instance).collect::<Vec<_>>();
    if errors.is_empty() {
        return Ok(());
    }

    let schema = crate::contract::find_tool(tool).map(|spec| spec.input_schema());
    let issues = schema
        .as_ref()
        .and_then(|schema| reference_issues(tool, schema, &errors));
    if let Some(issues) = issues {
        return Err(format!(
            "MCP error -32602: Input validation error: Invalid arguments for tool {tool}: {issues}"
        ));
    }

    let error = &errors[0];
    let path = error.instance_path();
    Err(format!(
        "MCP error -32602: Input validation error: Invalid arguments for tool {tool} at {path}: {error}"
    ))
}

fn reference_issues(
    tool: &str,
    schema: &Map<String, Value>,
    errors: &[jsonschema::ValidationError<'_>],
) -> Option<String> {
    let mut issues = errors
        .iter()
        .map(|error| reference_issue(schema, error))
        .collect::<Option<Vec<_>>>()?;

    // A Zod enum emits only `invalid_value`, while JSON Schema can additionally
    // emit a type error for the same non-string value.
    let enum_paths = issues
        .iter()
        .filter(|issue| issue.is_invalid_value())
        .map(|issue| issue.path().to_vec())
        .collect::<Vec<_>>();
    issues.retain(|issue| {
        !issue.is_invalid_type() || !enum_paths.iter().any(|path| path == issue.path())
    });

    let order = field_order(tool);
    issues.sort_by_key(|issue| {
        issue
            .path()
            .first()
            .and_then(|field| order.iter().position(|candidate| candidate == field))
            .unwrap_or(usize::MAX)
    });
    serde_json::to_string_pretty(&issues).ok()
}

fn reference_issue(
    schema: &Map<String, Value>,
    error: &jsonschema::ValidationError<'_>,
) -> Option<ReferenceIssue> {
    let path = pointer_path(&error.instance_path().to_string());
    match error.kind() {
        ValidationErrorKind::Required { property } => {
            let property = property.as_str()?.to_owned();
            let expected = property_type(schema, &property)?;
            let mut path = path;
            path.push(property);
            Some(ReferenceIssue::InvalidType(InvalidTypeIssue {
                expected: expected.to_owned(),
                format: None,
                code: "invalid_type",
                path,
                message: format!("Invalid input: expected {expected}, received undefined"),
            }))
        }
        ValidationErrorKind::Type { kind } => {
            let received = json_type(error.instance().as_ref());
            let (expected, format) = match kind {
                TypeKind::Single(JsonType::Integer) if error.instance().is_number() => {
                    ("int", Some("safeint"))
                }
                TypeKind::Single(JsonType::Integer) => ("number", None),
                TypeKind::Single(kind) => (kind.as_str(), None),
                TypeKind::Multiple(_) => return None,
            };
            Some(ReferenceIssue::InvalidType(InvalidTypeIssue {
                expected: expected.to_owned(),
                format,
                code: "invalid_type",
                path,
                message: format!("Invalid input: expected {expected}, received {received}"),
            }))
        }
        ValidationErrorKind::Enum { options } => {
            let values = options.as_array()?.clone();
            let choices = values
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join("|");
            Some(ReferenceIssue::InvalidValue(InvalidValueIssue {
                code: "invalid_value",
                values,
                path,
                message: format!("Invalid option: expected one of {choices}"),
            }))
        }
        ValidationErrorKind::Minimum { limit } => Some(range_issue(
            false,
            limit.clone(),
            path,
            is_safe_integer_limit(limit),
        )),
        ValidationErrorKind::Maximum { limit } => Some(range_issue(
            true,
            limit.clone(),
            path,
            is_safe_integer_limit(limit),
        )),
        _ => None,
    }
}

fn range_issue(
    maximum: bool,
    limit: Value,
    path: Vec<String>,
    safe_integer: bool,
) -> ReferenceIssue {
    let comparison = if maximum { "<=" } else { ">=" };
    let direction = if maximum { "Too big" } else { "Too small" };
    let origin = if safe_integer { "int" } else { "number" };
    let message = format!("{direction}: expected {origin} to be {comparison}{}", limit);
    match (maximum, safe_integer) {
        (true, true) => ReferenceIssue::SafeMaximum(SafeMaximumIssue {
            code: "too_big",
            maximum: limit,
            note: "Integers must be within the safe integer range.",
            origin,
            inclusive: true,
            path,
            message,
        }),
        (false, true) => ReferenceIssue::SafeMinimum(SafeMinimumIssue {
            code: "too_small",
            minimum: limit,
            note: "Integers must be within the safe integer range.",
            origin,
            inclusive: true,
            path,
            message,
        }),
        (true, false) => ReferenceIssue::Maximum(MaximumIssue {
            origin,
            code: "too_big",
            maximum: limit,
            inclusive: true,
            path,
            message,
        }),
        (false, false) => ReferenceIssue::Minimum(MinimumIssue {
            origin,
            code: "too_small",
            minimum: limit,
            inclusive: true,
            path,
            message,
        }),
    }
}

fn property_type<'a>(schema: &'a Map<String, Value>, property: &str) -> Option<&'a str> {
    let kind = schema
        .get("properties")?
        .get(property)?
        .get("type")?
        .as_str()?;
    Some(if kind == "integer" { "number" } else { kind })
}

fn json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn is_safe_integer_limit(value: &Value) -> bool {
    matches!(
        value.as_i64(),
        Some(-9_007_199_254_740_991 | 9_007_199_254_740_991)
    )
}

fn pointer_path(pointer: &str) -> Vec<String> {
    pointer
        .split('/')
        .skip(1)
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
        .collect()
}

fn field_order(tool: &str) -> &'static [&'static str] {
    match tool {
        "bazi_chart" | "bazi_structure" => &["datetime", "gender"],
        "bazi_timeline" => &["datetime", "gender", "startYear", "count"],
        "bazi_period_detail" => &[
            "datetime", "gender", "scope", "year", "month", "date", "hour",
        ],
        "bazi_shensha" => &["datetime", "onlyHits"],
        "ziwei_chart" => &[
            "datetime",
            "gender",
            "profile",
            "calendar",
            "isLeapMonth",
            "language",
        ],
        "ziwei_palace_detail" => &[
            "datetime",
            "gender",
            "palace",
            "profile",
            "calendar",
            "isLeapMonth",
            "language",
        ],
        "ziwei_horoscope_overview" => &[
            "birthDatetime",
            "gender",
            "targetDatetime",
            "profile",
            "calendar",
            "isLeapMonth",
            "language",
        ],
        "ziwei_scope_detail" => &[
            "birthDatetime",
            "gender",
            "targetDatetime",
            "scope",
            "focusPalace",
            "profile",
            "calendar",
            "isLeapMonth",
            "language",
        ],
        "ziwei_topic_context" => &[
            "birthDatetime",
            "gender",
            "targetDatetime",
            "topic",
            "profile",
            "calendar",
            "isLeapMonth",
            "language",
        ],
        _ => &[],
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum ReferenceIssue {
    InvalidType(InvalidTypeIssue),
    InvalidValue(InvalidValueIssue),
    Minimum(MinimumIssue),
    Maximum(MaximumIssue),
    SafeMinimum(SafeMinimumIssue),
    SafeMaximum(SafeMaximumIssue),
}

impl ReferenceIssue {
    fn path(&self) -> &[String] {
        match self {
            Self::InvalidType(issue) => &issue.path,
            Self::InvalidValue(issue) => &issue.path,
            Self::Minimum(issue) => &issue.path,
            Self::Maximum(issue) => &issue.path,
            Self::SafeMinimum(issue) => &issue.path,
            Self::SafeMaximum(issue) => &issue.path,
        }
    }

    fn is_invalid_type(&self) -> bool {
        matches!(self, Self::InvalidType(_))
    }

    fn is_invalid_value(&self) -> bool {
        matches!(self, Self::InvalidValue(_))
    }
}

#[derive(Serialize)]
struct InvalidTypeIssue {
    expected: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'static str>,
    code: &'static str,
    path: Vec<String>,
    message: String,
}

#[derive(Serialize)]
struct InvalidValueIssue {
    code: &'static str,
    values: Vec<Value>,
    path: Vec<String>,
    message: String,
}

#[derive(Serialize)]
struct MinimumIssue {
    origin: &'static str,
    code: &'static str,
    minimum: Value,
    inclusive: bool,
    path: Vec<String>,
    message: String,
}

#[derive(Serialize)]
struct MaximumIssue {
    origin: &'static str,
    code: &'static str,
    maximum: Value,
    inclusive: bool,
    path: Vec<String>,
    message: String,
}

#[derive(Serialize)]
struct SafeMinimumIssue {
    code: &'static str,
    minimum: Value,
    note: &'static str,
    origin: &'static str,
    inclusive: bool,
    path: Vec<String>,
    message: String,
}

#[derive(Serialize)]
struct SafeMaximumIssue {
    code: &'static str,
    maximum: Value,
    note: &'static str,
    origin: &'static str,
    inclusive: bool,
    path: Vec<String>,
    message: String,
}

fn build_validators() -> Result<HashMap<&'static str, jsonschema::Validator>, String> {
    crate::contract::tool_specs()
        .iter()
        .map(|spec| {
            let schema = Value::Object(spec.input_schema());
            jsonschema::validator_for(&schema)
                .map(|validator| (spec.name, validator))
                .map_err(|error| format!("invalid internal schema for {}: {error}", spec.name))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn validates_enums_and_numeric_ranges_from_registry() {
        let invalid_scope = json!({
            "datetime": "2024-01-15 08:30",
            "gender": "男",
            "scope": "week"
        });
        let error = arguments(
            "bazi_period_detail",
            invalid_scope.as_object().expect("object"),
        )
        .expect_err("scope must be rejected");
        assert!(error.contains("-32602"));
        assert!(error.contains("scope"), "{error}");

        let invalid_count = json!({
            "datetime": "2024-01-15 08:30",
            "gender": "男",
            "startYear": 2026,
            "count": 61
        });
        assert!(arguments("bazi_timeline", invalid_count.as_object().expect("object")).is_err());
    }
}

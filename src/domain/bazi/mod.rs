mod engine;
mod model;
mod render;
mod shensha;

use serde_json::{Map, Value};

fn string<'a>(arguments: &'a Map<String, Value>, name: &str) -> Result<&'a str, String> {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing or invalid argument: {name}"))
}

fn optional_integer(arguments: &Map<String, Value>, name: &str) -> Result<Option<isize>, String> {
    match arguments.get(name) {
        None => Ok(None),
        Some(value) => {
            let integer = value
                .as_i64()
                .and_then(|value| isize::try_from(value).ok())
                .or_else(|| value.as_u64().and_then(|value| isize::try_from(value).ok()))
                .or_else(|| {
                    let value = value.as_f64()?;
                    (value.is_finite()
                        && value.fract() == 0.0
                        && value >= isize::MIN as f64
                        && value <= isize::MAX as f64)
                        .then_some(value as isize)
                });
            integer
                .map(Some)
                .ok_or_else(|| format!("{name} must be an integer."))
        }
    }
}

/// Executes one of the five BaZi tools and returns its Markdown-only payload.
///
/// The MCP presentation layer adds the reference tool-name prefix to errors.
pub fn execute(tool: &str, arguments: &Map<String, Value>) -> Result<String, String> {
    let run = || -> Result<String, String> {
        match tool {
            "bazi_chart" => Ok(render::chart(&engine::build_chart(
                string(arguments, "datetime")?,
                string(arguments, "gender")?,
            )?)),
            "bazi_structure" => Ok(render::structure(&engine::build_structure(
                string(arguments, "datetime")?,
                string(arguments, "gender")?,
            )?)),
            "bazi_timeline" => {
                let start_year = optional_integer(arguments, "startYear")?
                    .ok_or_else(|| "missing or invalid argument: startYear".to_string())?;
                let count = optional_integer(arguments, "count")?.unwrap_or(10);
                if !(1..=60).contains(&count) {
                    return Err("count must be 1-60.".into());
                }
                Ok(render::timeline(&engine::build_timeline(
                    string(arguments, "datetime")?,
                    string(arguments, "gender")?,
                    start_year,
                    count as usize,
                )?))
            }
            "bazi_period_detail" => {
                let month = optional_integer(arguments, "month")?
                    .map(usize::try_from)
                    .transpose()
                    .map_err(|_| "month must be a non-negative integer.".to_owned())?;
                let hour = optional_integer(arguments, "hour")?
                    .map(usize::try_from)
                    .transpose()
                    .map_err(|_| "hour must be a non-negative integer.".to_owned())?;
                let date = arguments.get("date").and_then(Value::as_str);
                Ok(render::period(&engine::build_period(
                    string(arguments, "datetime")?,
                    string(arguments, "gender")?,
                    string(arguments, "scope")?,
                    optional_integer(arguments, "year")?,
                    month,
                    date,
                    hour,
                )?))
            }
            "bazi_shensha" => {
                let datetime = string(arguments, "datetime")?;
                if let Some(value) = arguments.get("onlyHits")
                    && !value.is_boolean()
                {
                    return Err("onlyHits must be a boolean.".into());
                }
                Ok(render::shensha(datetime, &engine::build_shensha(datetime)?))
            }
            _ => Err(format!("unknown BaZi tool: {tool}")),
        }
    };
    run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn args(value: Value) -> Map<String, Value> {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn all_tools_return_markdown() {
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
        ];
        for (tool, value, title) in cases {
            let text = execute(tool, &args(value)).unwrap();
            assert!(text.contains(title));
            assert!(text.contains("| "));
        }
    }

    #[test]
    fn applies_defaults_and_conditional_validation() {
        let timeline = execute(
            "bazi_timeline",
            &args(json!({"datetime":"2024-01-15 08:30","gender":"男","startYear":2026})),
        )
        .unwrap();
        assert!(timeline.contains("查询: 2026 起 10 年"));
        let error = execute("bazi_period_detail", &args(json!({"datetime":"2024-01-15 08:30","gender":"男","scope":"hour","date":"2026-06-12"}))).unwrap_err();
        assert!(error.contains("scope=hour requires hour 0-23"));
    }
}

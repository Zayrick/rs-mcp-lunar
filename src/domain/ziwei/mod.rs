mod engine;
mod locale;
mod render;

use serde_json::{Map, Value};

/// Executes one of the five Zi Wei Dou Shu tools and returns its Markdown body.
///
/// Input-schema ownership stays at the MCP registry boundary; this dispatcher
/// nevertheless validates required values so it remains safe to call directly
/// from tests or another transport adapter. Errors are intentionally returned
/// without a tool-name prefix—the MCP presentation layer adds that prefix once.
pub fn execute(tool: &str, arguments: &Map<String, Value>) -> Result<String, String> {
    match tool {
        "ziwei_chart" => {
            let args = engine::common_chart_args(arguments, "datetime")?;
            Ok(render::chart(&engine::build_chart(args)?))
        }
        "ziwei_palace_detail" => {
            let palace = engine::required_string(arguments, "palace")?.to_owned();
            let args = engine::common_chart_args(arguments, "datetime")?;
            render::palace_detail(&engine::build_chart(args)?, &palace)
        }
        "ziwei_horoscope_overview" => {
            let args = engine::horoscope_args(arguments)?;
            render::horoscope_overview(&engine::build_horoscope(args)?)
        }
        "ziwei_scope_detail" => {
            let scope = engine::normalize_scope(engine::required_string(arguments, "scope")?)?;
            let focus = optional_string(arguments, "focusPalace")?
                .unwrap_or("命宫")
                .to_owned();
            let args = engine::horoscope_args(arguments)?;
            render::scope_detail(&engine::build_horoscope(args)?, scope, &focus)
        }
        "ziwei_topic_context" => {
            let topic = engine::normalize_topic(engine::required_string(arguments, "topic")?)?;
            let args = engine::horoscope_args(arguments)?;
            render::topic_context(&engine::build_horoscope(args)?, topic)
        }
        _ => Err(format!("Unknown Ziwei tool: {tool}")),
    }
}

fn optional_string<'a>(
    arguments: &'a Map<String, Value>,
    key: &str,
) -> Result<Option<&'a str>, String> {
    match arguments.get(key) {
        Some(Value::String(value)) => Ok(Some(value)),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(format!("Invalid {key}. Expected a string.")),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn arguments(value: Value) -> Map<String, Value> {
        value.as_object().expect("test input is an object").clone()
    }

    #[test]
    fn dispatches_all_five_tools() {
        let natal = arguments(json!({"datetime":"2024-01-15 08:30","gender":"男"}));
        let palace = arguments(json!({
            "datetime":"2024-01-15 08:30","gender":"男","palace":"命宫"
        }));
        let runtime = arguments(json!({
            "birthDatetime":"2024-01-15 08:30",
            "gender":"男",
            "targetDatetime":"2026-06-12 18:00"
        }));
        let mut scope = runtime.clone();
        scope.insert("scope".into(), json!("yearly"));
        let mut topic = runtime.clone();
        topic.insert("topic".into(), json!("career"));

        assert!(
            execute("ziwei_chart", &natal)
                .unwrap()
                .contains("紫微斗数本命全盘")
        );
        assert!(
            execute("ziwei_palace_detail", &palace)
                .unwrap()
                .contains("紫微斗数单宫详盘")
        );
        assert!(
            execute("ziwei_horoscope_overview", &runtime)
                .unwrap()
                .contains("紫微运限总览")
        );
        assert!(
            execute("ziwei_scope_detail", &scope)
                .unwrap()
                .contains("紫微单层运限详盘")
        );
        let topic_output = execute("ziwei_topic_context", &topic).unwrap();
        assert!(topic_output.contains("紫微专题取证"));
        assert!(topic_output.len() < 6_000);
        assert!(topic_output.contains("ziwei_palace_detail(palace=官禄)"));
    }

    #[test]
    fn errors_are_unprefixed_for_mcp_layer() {
        let args = arguments(json!({
            "datetime":"2024-01-15 08:30","gender":"男","profile":"legacy"
        }));
        let error = execute("ziwei_chart", &args).unwrap_err();
        assert_eq!(error, "Unsupported profile. Use sanhe or feixing-sihua.");
        assert!(!error.starts_with("ziwei_chart 错误:"));
    }

    #[test]
    fn non_chinese_runtime_palace_lookup_uses_typed_identity() {
        // iztro@2.5.8 resolves runtime palace names through localized strings
        // and throws `invalid palace index` outside zh-CN. Keeping the palace
        // identity typed fixes that upstream presentation-layer bug.
        let scope = arguments(json!({
            "birthDatetime":"2024-01-15 08:30",
            "gender":"男",
            "targetDatetime":"2026-06-12 18:00",
            "scope":"yearly",
            "language":"en-US"
        }));
        let topic = arguments(json!({
            "birthDatetime":"2024-01-15 08:30",
            "gender":"男",
            "targetDatetime":"2026-06-12 18:00",
            "topic":"career",
            "language":"en-US"
        }));

        assert!(
            execute("ziwei_scope_detail", &scope)
                .unwrap()
                .contains("soul")
        );
        assert!(
            execute("ziwei_topic_context", &topic)
                .unwrap()
                .contains("career")
        );
    }

    #[test]
    fn supports_childhood_and_late_half_leap_month_runtime() {
        let args = arguments(json!({
            "birthDatetime":"2023-02-20 23:30",
            "gender":"女",
            "profile":"sanhe",
            "calendar":"lunar",
            "isLeapMonth":true,
            "targetDatetime":"2026-06-12 18:00"
        }));

        let output = execute("ziwei_horoscope_overview", &args).unwrap();
        assert!(output.contains("| 大限 | 甲寅 | - | 夫妻(甲寅) |"));
        assert!(output.contains("| 流月 | 癸巳 |"));
        assert!(output.contains("| 田宅(己未) |"));
    }

    #[test]
    fn feixing_runtime_uses_lichun_year_inside_cny_gap() {
        let args = arguments(json!({
            "birthDatetime":"2000-02-29 23:30",
            "gender":"男",
            "profile":"feixing-sihua",
            "targetDatetime":"2026-02-10 18:00"
        }));

        let output = execute("ziwei_horoscope_overview", &args).unwrap();
        assert!(output.contains("| 流年 | 丙午 |"));
        assert!(output.contains("| 流月 | 庚寅 |"));
        assert!(output.contains("| 流时 | 乙酉 |"));
    }

    #[test]
    fn out_of_library_solar_year_is_a_result_not_a_panic() {
        let cases = [
            (
                "ziwei_chart",
                json!({"datetime":"0000-01-01 00:00","gender":"男"}),
            ),
            (
                "ziwei_chart",
                json!({"datetime":"9999-12-31 23:30","gender":"男"}),
            ),
            (
                "ziwei_chart",
                json!({"datetime":"9999-12-31 23:30","gender":"男","profile":"feixing-sihua"}),
            ),
            (
                "ziwei_chart",
                json!({"datetime":"0000-01-01 00:00","gender":"男","calendar":"lunar"}),
            ),
            (
                "ziwei_chart",
                json!({"datetime":"2024-13-01 00:00","gender":"男"}),
            ),
            (
                "ziwei_chart",
                json!({"datetime":"2024-02-30 00:00","gender":"男"}),
            ),
            (
                "ziwei_chart",
                json!({"datetime":"2024-13-01 00:00","gender":"男","calendar":"lunar"}),
            ),
            (
                "ziwei_chart",
                json!({"datetime":"2024-02-30 00:00","gender":"男","calendar":"lunar"}),
            ),
            (
                "ziwei_horoscope_overview",
                json!({
                    "birthDatetime":"2024-01-15 08:30",
                    "gender":"男",
                    "targetDatetime":"0000-01-01 00:00"
                }),
            ),
            (
                "ziwei_horoscope_overview",
                json!({
                    "birthDatetime":"2024-01-15 08:30",
                    "gender":"男",
                    "targetDatetime":"9999-12-31 23:30",
                    "profile":"feixing-sihua"
                }),
            ),
        ];
        for (tool, value) in cases {
            let args = arguments(value);
            let result = std::panic::catch_unwind(|| execute(tool, &args));
            assert!(
                result.is_ok(),
                "{tool} invalid calendar input must not unwind"
            );
        }
    }

    #[test]
    fn horoscope_range_boundary_is_an_error_not_a_panic() {
        for profile in ["sanhe", "feixing-sihua"] {
            let args = arguments(json!({
                "birthDatetime":"9998-01-01 12:00",
                "gender":"男",
                "targetDatetime":"9999-12-31 23:30",
                "profile":profile
            }));
            let result = std::panic::catch_unwind(|| execute("ziwei_horoscope_overview", &args));
            assert!(result.is_ok(), "{profile} target boundary unwound");
            assert_eq!(
                result.expect("catch_unwind checked").unwrap_err(),
                "targetDatetime exceeds tyme4rs safe range (year 2..=9998)."
            );
        }
    }

    #[test]
    fn solar_birth_year_zero_boundary_is_an_error_not_a_panic() {
        for profile in ["sanhe", "feixing-sihua"] {
            let args = arguments(json!({
                "datetime":"0001-01-01 00:00",
                "gender":"男",
                "profile":profile
            }));
            let result = std::panic::catch_unwind(|| execute("ziwei_chart", &args));
            assert!(result.is_ok(), "{profile} solar birth boundary unwound");
            assert_eq!(
                result.expect("catch_unwind checked").unwrap_err(),
                "solar datetime exceeds tyme4rs safe range (year 2..=9999)."
            );
        }
    }

    #[test]
    fn supported_horoscope_calendar_edges_do_not_unwind() {
        let cases = [
            ("0002-01-01 00:00", "0002-12-31 23:30", "solar"),
            ("9998-01-01 00:00", "9998-12-31 23:30", "solar"),
            ("0001-01-01 00:00", "0002-12-31 23:30", "lunar"),
            ("9998-01-01 00:00", "9998-12-31 23:30", "lunar"),
        ];
        for profile in ["sanhe", "feixing-sihua"] {
            for (birth, target, calendar) in cases {
                let args = arguments(json!({
                    "birthDatetime":birth,
                    "gender":"男",
                    "targetDatetime":target,
                    "profile":profile,
                    "calendar":calendar
                }));
                let result =
                    std::panic::catch_unwind(|| execute("ziwei_horoscope_overview", &args));
                assert!(
                    result.is_ok(),
                    "{profile}/{calendar} supported edge unwound: {birth} -> {target}"
                );
                assert!(
                    result.expect("catch_unwind checked").is_ok(),
                    "{profile}/{calendar} supported edge returned an error: {birth} -> {target}"
                );
            }
        }
    }
}

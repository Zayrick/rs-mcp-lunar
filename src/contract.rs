use serde_json::{Map, Value, json};

pub type JsonObject = Map<String, Value>;

pub const SERVER_NAME: &str = "Lunar Calendar MCP";
pub const SERVER_VERSION: &str = "1.0.0";
pub const MCP_PATH: &str = "/lunar";

#[derive(Clone, Copy)]
enum SchemaKind {
    BaziChart,
    BaziStructure,
    BaziTimeline,
    BaziPeriod,
    BaziShensha,
    ZiweiChart,
    ZiweiPalace,
    ZiweiOverview,
    ZiweiScope,
    ZiweiTopic,
}

#[derive(Clone, Copy)]
pub struct ToolSpec {
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    schema: SchemaKind,
}

const TOOL_SPECS: [ToolSpec; 10] = [
    ToolSpec {
        name: "bazi_chart",
        title: "八字排盘",
        description: "适用场景: 第一步获取八字客观基础盘。不要用于直接判断身强身弱、格局或用神。下一步: 调用 bazi_structure 做命局结构取证，或 bazi_timeline 看阶段触发。",
        schema: SchemaKind::BaziChart,
    },
    ToolSpec {
        name: "bazi_structure",
        title: "八字命局分析",
        description: "适用场景: 在 bazi_chart 后分析日主、月令、通根、透干、五行、十神、刑冲合害与旺衰取用证据。不要输出最终断命结论。下一步: 调用 bazi_timeline 或 bazi_period_detail 验证阶段。",
        schema: SchemaKind::BaziStructure,
    },
    ToolSpec {
        name: "bazi_timeline",
        title: "八字大运流年",
        description: "适用场景: 在本命结构后查看大运、流年、小运、年龄、年份和原局触发。不要用于单独断某一年细节。下一步: 对重点年份调用 bazi_period_detail。",
        schema: SchemaKind::BaziTimeline,
    },
    ToolSpec {
        name: "bazi_period_detail",
        title: "八字周期详盘",
        description: "适用场景: 展开某一年、干支月、日或小时与原局/大运流年的叠加证据。不要替代 bazi_structure 的命局结构判断。下一步: 回到用户专题问题组织解读。",
        schema: SchemaKind::BaziPeriod,
    },
    ToolSpec {
        name: "bazi_shensha",
        title: "八字神煞参考",
        description: "适用场景: 需要神煞作为附加证据时调用。不要单独用神煞断事或替代 bazi_structure。下一步: 回到 bazi_structure 或 bazi_period_detail 与主结构合看。",
        schema: SchemaKind::BaziShensha,
    },
    ToolSpec {
        name: "ziwei_chart",
        title: "紫微斗数排盘",
        description: "适用场景: 第一步获取紫微本命十二宫全盘。不要用于展开单宫飞化或运限叠盘。下一步: 调用 ziwei_palace_detail 看单宫，或 ziwei_horoscope_overview 看运限。",
        schema: SchemaKind::ZiweiChart,
    },
    ToolSpec {
        name: "ziwei_palace_detail",
        title: "紫微宫位详盘",
        description: "适用场景: 展开某个本命宫位的本宫、对宫、三方四正、夹宫、空宫借星、飞化和自化证据。不要用于运限总览。下一步: 要叠运限调用 ziwei_scope_detail 或 ziwei_topic_context。",
        schema: SchemaKind::ZiweiPalace,
    },
    ToolSpec {
        name: "ziwei_horoscope_overview",
        title: "紫微运限概览",
        description: "适用场景: 只做大限、小限、流年、流月、流日、流时入口级导航。不要作为最终运势分析。下一步: 必须调用 ziwei_scope_detail 展开单层，或 ziwei_topic_context 做专题取证。",
        schema: SchemaKind::ZiweiOverview,
    },
    ToolSpec {
        name: "ziwei_scope_detail",
        title: "紫微运限详盘",
        description: "适用场景: 展开一个层级的大限/小限/流年/流月/流日/流时十二宫映射、流耀、四化与重点宫位三方四正。不要一次请求全部层级。下一步: 专题整合调用 ziwei_topic_context。",
        schema: SchemaKind::ZiweiScope,
    },
    ToolSpec {
        name: "ziwei_topic_context",
        title: "紫微专题分析",
        description: "适用场景: 针对自我、事业、财富、关系、健康、家庭聚合本命与流年证据。不要直接输出最终断语。下一步: 对关键宫位调用 ziwei_palace_detail 或 ziwei_scope_detail。",
        schema: SchemaKind::ZiweiTopic,
    },
];

pub fn tool_specs() -> &'static [ToolSpec] {
    &TOOL_SPECS
}

pub fn find_tool(name: &str) -> Option<&'static ToolSpec> {
    TOOL_SPECS.iter().find(|spec| spec.name == name)
}

impl ToolSpec {
    pub fn input_schema(&self) -> JsonObject {
        schema(self.schema)
    }
}

pub fn server_info_text() -> String {
    let titles = TOOL_SPECS
        .iter()
        .map(|spec| spec.title)
        .collect::<Vec<_>>()
        .join("、");
    let rows = TOOL_SPECS
        .iter()
        .map(|spec| format!("| {} | {} |", spec.name, spec.title))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "Lunar Calendar MCP Server\n\n{titles}。\n\nMCP endpoint: {MCP_PATH}\n\n| Tool | Title |\n| --- | --- |\n{rows}"
    )
}

fn object(value: Value) -> JsonObject {
    // Every caller is a fixed `json!({ ... })` literal, and contract tests
    // assert the resulting schemas. A defensive default keeps this registry
    // total if a future edit accidentally changes that internal invariant.
    value.as_object().cloned().unwrap_or_default()
}

fn schema(kind: SchemaKind) -> JsonObject {
    match kind {
        SchemaKind::BaziChart => object(json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "datetime": {"type": "string", "description": "出生日期时间 YYYY-MM-DD HH:MM，按出生地当地民用时间输入"},
                "gender": {"type": "string", "description": "性别: 男/女 或 male/female"}
            },
            "required": ["datetime", "gender"]
        })),
        SchemaKind::BaziStructure => object(json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "datetime": {"type": "string", "description": "出生日期时间 YYYY-MM-DD HH:MM"},
                "gender": {"type": "string", "description": "性别: 男/女 或 male/female"}
            },
            "required": ["datetime", "gender"]
        })),
        SchemaKind::BaziTimeline => object(json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "datetime": {"type": "string", "description": "出生日期时间 YYYY-MM-DD HH:MM"},
                "gender": {"type": "string", "description": "性别: 男/女 或 male/female"},
                "startYear": {"type": "integer", "minimum": -9007199254740991_i64, "maximum": 9007199254740991_i64, "description": "起始公历年份"},
                "count": {"type": "integer", "minimum": 1, "maximum": 60, "default": 10, "description": "查询年数，1-60，默认10"}
            },
            "required": ["datetime", "gender", "startYear"]
        })),
        SchemaKind::BaziPeriod => object(json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "datetime": {"type": "string", "description": "出生日期时间 YYYY-MM-DD HH:MM"},
                "gender": {"type": "string", "description": "性别: 男/女 或 male/female"},
                "scope": {"type": "string", "enum": ["year", "month", "day", "hour"], "description": "周期层级: year/month/day/hour"},
                "year": {"type": "integer", "minimum": -9007199254740991_i64, "maximum": 9007199254740991_i64, "description": "scope=year/month 时必填，公历年份"},
                "month": {"type": "integer", "minimum": 1, "maximum": 12, "description": "scope=month 时必填，干支月序号 1-12"},
                "date": {"type": "string", "description": "scope=day/hour 时必填，YYYY-MM-DD"},
                "hour": {"type": "integer", "minimum": 0, "maximum": 23, "description": "scope=hour 时必填，0-23"}
            },
            "required": ["datetime", "gender", "scope"]
        })),
        SchemaKind::BaziShensha => object(json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "datetime": {"type": "string", "description": "出生日期时间 YYYY-MM-DD HH:MM"},
                "onlyHits": {"type": "boolean", "default": true, "description": "是否只返回命中项。当前按 taibu-core 神煞命中清单输出，默认 true。"}
            },
            "required": ["datetime"]
        })),
        SchemaKind::ZiweiChart => ziwei_schema(
            vec![
                (
                    "datetime",
                    string("出生日期时间 YYYY-MM-DD HH:MM；calendar=lunar 时日期部分按农历解释"),
                ),
                ("gender", string("性别: 男/女 或 male/female")),
            ],
            &["datetime", "gender"],
            ZiweiCommon::Chart,
        ),
        SchemaKind::ZiweiPalace => ziwei_schema(
            vec![
                (
                    "datetime",
                    string("出生日期时间 YYYY-MM-DD HH:MM；calendar=lunar 时日期部分按农历解释"),
                ),
                ("gender", string("性别: 男/女 或 male/female")),
                (
                    "palace",
                    string("宫位名称，如 命宫、夫妻、财帛、官禄、疾厄"),
                ),
            ],
            &["datetime", "gender", "palace"],
            ZiweiCommon::Palace,
        ),
        SchemaKind::ZiweiOverview => ziwei_schema(
            vec![
                (
                    "birthDatetime",
                    string("出生日期时间 YYYY-MM-DD HH:MM；calendar=lunar 时日期部分按农历解释"),
                ),
                ("gender", string("性别: 男/女 或 male/female")),
                ("targetDatetime", string("目标日期时间 YYYY-MM-DD HH:MM")),
            ],
            &["birthDatetime", "gender", "targetDatetime"],
            ZiweiCommon::Runtime,
        ),
        SchemaKind::ZiweiScope => ziwei_schema(
            vec![
                (
                    "birthDatetime",
                    string("出生日期时间 YYYY-MM-DD HH:MM；calendar=lunar 时日期部分按农历解释"),
                ),
                ("gender", string("性别: 男/女 或 male/female")),
                ("targetDatetime", string("目标日期时间 YYYY-MM-DD HH:MM")),
                (
                    "scope",
                    json!({"type": "string", "enum": ["decadal", "age", "yearly", "monthly", "daily", "hourly"], "description": "层级: decadal/age/yearly/monthly/daily/hourly"}),
                ),
                (
                    "focusPalace",
                    json!({"type": "string", "default": "命宫", "description": "重点宫位，默认命宫；age 层级只返回小限宫位"}),
                ),
            ],
            &["birthDatetime", "gender", "targetDatetime", "scope"],
            ZiweiCommon::Runtime,
        ),
        SchemaKind::ZiweiTopic => ziwei_schema(
            vec![
                (
                    "birthDatetime",
                    string("出生日期时间 YYYY-MM-DD HH:MM；calendar=lunar 时日期部分按农历解释"),
                ),
                ("gender", string("性别: 男/女 或 male/female")),
                ("targetDatetime", string("目标日期时间 YYYY-MM-DD HH:MM")),
                (
                    "topic",
                    json!({"type": "string", "enum": ["self", "career", "wealth", "relationship", "health", "family"], "description": "专题: self/career/wealth/relationship/health/family"}),
                ),
            ],
            &["birthDatetime", "gender", "targetDatetime", "topic"],
            ZiweiCommon::Runtime,
        ),
    }
}

fn string(description: &'static str) -> Value {
    json!({"type": "string", "description": description})
}

#[derive(Clone, Copy)]
enum ZiweiCommon {
    Chart,
    Palace,
    Runtime,
}

fn ziwei_schema(
    base: Vec<(&'static str, Value)>,
    required: &[&str],
    common: ZiweiCommon,
) -> JsonObject {
    let mut properties = Map::new();
    for (name, value) in base {
        properties.insert(name.to_owned(), value);
    }
    let (profile_description, calendar_description, leap_description) = match common {
        ZiweiCommon::Chart => (
            "排盘配置: sanhe（三合）或 feixing-sihua（飞星四化）",
            "输入日期历法: solar=公历，lunar=农历",
            "calendar=lunar 时是否为农历闰月",
        ),
        ZiweiCommon::Palace => (
            "排盘配置: sanhe 或 feixing-sihua",
            "出生日期历法",
            "calendar=lunar 时是否为农历闰月",
        ),
        ZiweiCommon::Runtime => (
            "排盘配置: sanhe 或 feixing-sihua",
            "出生日期历法",
            "calendar=lunar 时出生日期是否为农历闰月",
        ),
    };
    properties.insert(
        "profile".into(),
        json!({"default": "sanhe", "description": profile_description, "type": "string", "enum": ["sanhe", "feixing-sihua"]}),
    );
    properties.insert(
        "calendar".into(),
        json!({"default": "solar", "description": calendar_description, "type": "string", "enum": ["solar", "lunar"]}),
    );
    properties.insert(
        "isLeapMonth".into(),
        json!({"default": false, "description": leap_description, "type": "boolean"}),
    );
    properties.insert(
        "language".into(),
        json!({"default": "zh-CN", "description": "输出语言，默认 zh-CN", "type": "string", "enum": ["zh-CN", "zh-TW", "en-US", "ja-JP", "ko-KR", "vi-VN"]}),
    );

    let mut result = Map::new();
    result.insert(
        "$schema".into(),
        Value::String("http://json-schema.org/draft-07/schema#".into()),
    );
    result.insert("type".into(), Value::String("object".into()));
    result.insert("properties".into(), Value::Object(properties));
    result.insert(
        "required".into(),
        Value::Array(
            required
                .iter()
                .map(|name| Value::String((*name).to_owned()))
                .collect(),
        ),
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_reference_order_and_no_output_schema() {
        let tools = tool_specs();
        let names = tools.iter().map(|tool| tool.name).collect::<Vec<_>>();
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
        assert_eq!(tools.len(), 10);
    }

    #[test]
    fn info_is_generated_from_registry() {
        let info = server_info_text();
        for spec in TOOL_SPECS {
            assert!(info.contains(spec.name));
            assert!(info.contains(spec.title));
        }
        assert!(info.contains("MCP endpoint: /lunar"));
    }
}

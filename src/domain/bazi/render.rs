use super::model::*;

const PILLAR_LABELS: [&str; 4] = ["年柱", "月柱", "日柱", "时柱"];

fn md_value(value: impl ToString) -> String {
    let value = value.to_string();
    let value = if value.is_empty() { "-" } else { &value };
    value
        .replace("\r\n", "<br>")
        .replace('\n', "<br>")
        .replace('|', "\\|")
}

fn table(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let mut output = vec![
        format!(
            "| {} |",
            headers.iter().map(md_value).collect::<Vec<_>>().join(" | ")
        ),
        format!(
            "| {} |",
            headers
                .iter()
                .map(|_| "---")
                .collect::<Vec<_>>()
                .join(" | ")
        ),
    ];
    output.extend(rows.into_iter().map(|row| {
        format!(
            "| {} |",
            row.into_iter()
                .map(md_value)
                .collect::<Vec<_>>()
                .join(" | ")
        )
    }));
    output.join("\n")
}

fn join(sections: Vec<Option<String>>) -> String {
    sections
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn hidden_text(stems: &[HiddenStem]) -> String {
    let text = stems
        .iter()
        .map(|item| format!("{}/{}/{}", item.stem, item.qi_type, item.ten_god))
        .collect::<Vec<_>>()
        .join("、");
    if text.is_empty() { "-".into() } else { text }
}

fn shensha_text(items: &[String]) -> String {
    if items.is_empty() {
        "-".into()
    } else {
        items.join("、")
    }
}

fn fmt_num(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if rounded.fract().abs() < 1e-9 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    }
}

pub(crate) fn chart(data: &ChartData) -> String {
    let pillar_rows = data
        .pillars
        .iter()
        .enumerate()
        .map(|(index, p)| {
            vec![
                PILLAR_LABELS[index].into(),
                p.pillar.clone(),
                p.ten_god.clone(),
                p.heaven_stem.clone(),
                p.earth_branch.clone(),
                hidden_text(&p.hidden_stems),
                p.terrain.clone(),
                p.self_sitting.clone(),
                p.kong_wang.clone(),
                p.nayin.clone(),
                shensha_text(&p.shensha),
            ]
        })
        .collect();
    let luck_rows = data
        .luck
        .iter()
        .map(|luck| {
            vec![
                luck.label.clone(),
                luck.cycle.clone().unwrap_or_default(),
                luck.ten_god.clone().unwrap_or_default(),
                luck.nayin.clone().unwrap_or_default(),
                format!("{}-{}岁", luck.start_age, luck.end_age),
                format!("{}-{}", luck.start_year, luck.end_year),
            ]
        })
        .collect();
    join(vec![
        Some("八字本命基础盘".into()),
        Some(format!("输入: {}\n性别: {}\n公历: {}\n农历: {}\n四柱: {}\n日主: {}\n日柱空亡: {}\n司令: {}",
            data.input, data.gender, data.solar, data.lunar, data.four_pillar_text, data.day_master,
            data.day_kong_wang, data.commander)),
        Some(table(&["柱", "干支", "十神", "天干", "地支", "藏干", "星运", "自坐", "空亡", "纳音", "神煞"], pillar_rows)),
        Some(table(&["节气", "时间"], vec![
            vec![format!("当前{}", data.current_jie.name), data.current_jie.time.clone()],
            vec![format!("下个{}", data.next_jie.name), data.next_jie.time.clone()],
        ])),
        Some(table(&["胎元", "胎息", "命宫", "身宫"], vec![vec![data.fetal_origin.clone(), data.fetal_breath.clone(), data.life_palace.clone(), data.body_palace.clone()]])),
        Some(format!("起运: {}\n起运日期: {}\n顺逆: {}", data.start_luck_description, data.start_luck_date, if data.is_forward { "顺行" } else { "逆行" })),
        Some(table(&["运", "干支", "十神", "纳音", "年龄", "年份"], luck_rows)),
        Some("边界: 此工具只提供本命基础盘。判断旺衰取用请继续调用 bazi_structure；看阶段触发请调用 bazi_timeline 或 bazi_period_detail。".into()),
    ])
}

fn relation_table(rows: &[RelationRow]) -> String {
    if rows.is_empty() {
        return "刑冲合害: 未见明显天干五合、六合、三合、三会、六冲、六害、相刑、破。".into();
    }
    table(
        &["类型", "关系"],
        rows.iter()
            .map(|r| vec![r.kind.clone(), r.relation.clone()])
            .collect(),
    )
}

pub(crate) fn structure(data: &StructureData) -> String {
    let checkpoints = vec![
        vec![
            "月令".into(),
            format!(
                "月支{}，{}",
                data.month_branch,
                season_hint(&data.month_branch)
            ),
        ],
        vec![
            "通根".into(),
            "查看四支藏干中是否有日主同五行或同干根气。".into(),
        ],
        vec![
            "透干".into(),
            "查看年月时天干十神是否帮扶、耗泄、克制或生扶日主。".into(),
        ],
        vec![
            "组合".into(),
            if data.relations.is_empty() {
                "原局未见明显合冲刑害触发点。".into()
            } else {
                "原局存在合冲刑害触发点，需看位置与远近。".into()
            },
        ],
        vec![
            "取用边界".into(),
            "此工具只给证据，不直接判定身强身弱、格局、用神或忌神。".into(),
        ],
    ];
    join(vec![
        Some("八字命局结构证据".into()),
        Some(format!(
            "输入: {}\n日主: {}\n月令: {}({})\n说明: 此工具不输出最终断命结论，不直接判定身强身弱、格局或用神。",
            data.chart.input, data.chart.day_master, data.month_branch, data.month_element
        )),
        Some(table(
            &["五行", "分数", "占比", "日主五行"],
            data.element_scores
                .iter()
                .map(|s| {
                    vec![
                        s.element.into(),
                        fmt_num(s.score),
                        format!("{}%", fmt_num(s.percent)),
                        if s.is_day_master {
                            "是".into()
                        } else {
                            String::new()
                        },
                    ]
                })
                .collect(),
        )),
        Some(table(
            &["十神", "权重"],
            data.ten_gods
                .iter()
                .map(|(name, score)| vec![name.clone(), fmt_num(*score)])
                .collect(),
        )),
        Some(table(
            &["柱", "地支", "同五行根气", "同干根气", "命中藏干"],
            data.roots
                .iter()
                .map(|r| {
                    vec![
                        r.pillar.into(),
                        r.branch.clone(),
                        if r.matches.is_empty() {
                            "无".into()
                        } else {
                            "有".into()
                        },
                        if r.exact { "有".into() } else { "无".into() },
                        hidden_text(&r.matches),
                    ]
                })
                .collect(),
        )),
        Some(table(
            &["透干位置", "天干", "十神"],
            data.visible_stems
                .iter()
                .map(|(p, s, t)| vec![(*p).into(), s.clone(), t.clone()])
                .collect(),
        )),
        Some(relation_table(&data.relations)),
        Some(table(&["检查项", "证据"], checkpoints)),
        Some(
            "下一步: 看阶段触发调用 bazi_timeline；看单一年/月/日/时调用 bazi_period_detail。"
                .into(),
        ),
    ])
}

fn season_hint(branch: &str) -> &'static str {
    if ["寅", "卯", "辰"].contains(&branch) {
        "春令木气，需结合透干、通根、寒暖燥湿继续判断。"
    } else if ["巳", "午", "未"].contains(&branch) {
        "夏令火气，需结合燥热、调候与泄耗继续判断。"
    } else if ["申", "酉", "戌"].contains(&branch) {
        "秋令金气，需结合肃杀、通关与扶抑继续判断。"
    } else {
        "冬令水气，需结合寒湿、调候与通根继续判断。"
    }
}

fn decade_text(decade: &Option<DecadeEvidence>) -> String {
    decade
        .as_ref()
        .map(|d| {
            format!(
                "{}({}) {}-{}岁",
                d.evidence.cycle, d.evidence.ten_god, d.start_age, d.end_age
            )
        })
        .unwrap_or_else(|| "-".into())
}

pub(crate) fn timeline(data: &TimelineData) -> String {
    join(vec![
        Some("八字大运流年时间轴".into()),
        Some(format!("出生: {}\n日主: {}\n查询: {} 起 {} 年", data.chart.input, data.chart.day_master, data.start_year, data.count)),
        Some(table(&["年份", "公历实际对应范围", "年龄", "阶段", "大运", "小运", "流年", "流年星运", "流年纳音", "太岁"], data.rows.iter().map(|r| vec![
            r.year.to_string(), r.solar_range.clone(), format!("{}岁", r.age), if r.decade.is_some() { "大运".into() } else { "起运前".into() },
            decade_text(&r.decade), format!("{}({})", r.small_luck.cycle, r.small_luck.ten_god),
            format!("{}({})", r.annual.cycle, r.annual.ten_god), r.annual.star_fortune.clone(), r.annual.nayin.clone(),
            if r.tai_sui.is_empty() { "-".into() } else { r.tai_sui.join("、") },
        ]).collect())),
        Some(table(&["年份", "流年", "与原局四柱关系"], data.rows.iter().map(|r| vec![
            r.year.to_string(), r.annual.cycle.clone(), r.original_relations.iter().map(|(p,o,x)| format!("{p}{o}:{x}")).collect::<Vec<_>>().join("；")
        ]).collect())),
        Some(table(&["年份", "流年", "胎元/命宫/身宫关系"], data.rows.iter().map(|r| vec![r.year.to_string(), r.annual.cycle.clone(), r.special_relations.clone()]).collect())),
        Some(table(&["年份", "大运神煞", "小运神煞", "流年神煞"], data.rows.iter().map(|r| vec![
            r.year.to_string(), r.decade.as_ref().map(|d| shensha_text(&d.evidence.shensha)).unwrap_or_else(|| "-".into()),
            shensha_text(&r.small_luck.shensha), shensha_text(&r.annual.shensha)
        ]).collect())),
        Some("边界: 此工具只给阶段时间轴和触发关系。若要展开某个年/月/日/时，请继续调用 bazi_period_detail。".into()),
    ])
}

pub(crate) fn period(data: &PeriodData) -> String {
    let e = &data.evidence;
    let details = table(
        &["项目", "值"],
        vec![
            vec!["天干".into(), e.heaven_stem.clone()],
            vec!["地支".into(), e.earth_branch.clone()],
            vec!["藏干".into(), hidden_text(&e.hidden_stems)],
            vec!["星运".into(), e.star_fortune.clone()],
            vec!["自坐".into(), e.self_sitting.clone()],
            vec!["空亡".into(), e.kong_wang.clone()],
            vec!["纳音".into(), e.nayin.clone()],
            vec!["神煞".into(), shensha_text(&e.shensha)],
        ],
    );
    let timeline = data.timeline.as_ref().map(|r| {
        table(
            &["年份", "年龄", "大运", "小运", "流年", "胎元/命宫/身宫关系"],
            vec![vec![
                r.year.to_string(),
                format!("{}岁", r.age),
                decade_text(&r.decade),
                format!("{}({})", r.small_luck.cycle, r.small_luck.ten_god),
                format!("{}({})", r.annual.cycle, r.annual.ten_god),
                r.special_relations.clone(),
            ]],
        )
    });
    let duty = if data.twelve_star.is_some() || data.nine_star.is_some() {
        Some(format!(
            "十二建星: {}\n九星: {}",
            data.twelve_star.as_deref().unwrap_or("-"),
            data.nine_star.as_deref().unwrap_or("-")
        ))
    } else {
        None
    };
    let recommends = data.recommends.as_ref().map(|yes| {
        table(
            &["宜", "忌"],
            vec![vec![
                yes.join("、"),
                data.avoids
                    .as_ref()
                    .map(|v| v.join("、"))
                    .unwrap_or_default(),
            ]],
        )
    });
    join(vec![
        Some("八字单一周期详盘".into()),
        Some(format!(
            "出生: {}\n四柱: {}\n日主: {}\n周期层级: {}\n周期: {}\n干支: {}({})\n星运: {}\n自坐: {}\n空亡: {}\n纳音: {}\n公历实际对应范围: {}",
            data.chart.input,
            data.chart.four_pillar_text,
            data.chart.day_master,
            data.scope,
            data.label,
            e.cycle,
            e.ten_god,
            e.star_fortune,
            e.self_sitting,
            if e.kong_wang.is_empty() {
                "-"
            } else {
                &e.kong_wang
            },
            e.nayin,
            data.solar_range
        )),
        data.jie_qi.as_ref().map(|v| format!("节气: {v}")),
        duty,
        timeline,
        Some(details),
        Some(table(
            &["原局柱", "原局干支", "与周期关系"],
            data.relations
                .iter()
                .map(|(p, o, r)| vec![(*p).into(), o.clone(), r.clone()])
                .collect(),
        )),
        data.special_relations
            .as_ref()
            .map(|v| format!("胎元/命宫/身宫关系: {v}")),
        recommends,
        Some("边界: 此工具只展开指定周期证据，不替代 bazi_structure 的命局结构判断。".into()),
    ])
}

pub(crate) fn shensha(datetime: &str, rows: &[(String, String, String)]) -> String {
    let body = if rows.is_empty() {
        "命中神煞: 无".into()
    } else {
        table(
            &["神煞", "命中位置", "干支"],
            rows.iter()
                .map(|(a, b, c)| vec![a.clone(), b.clone(), c.clone()])
                .collect(),
        )
    };
    join(vec![
        Some("常用八字神煞辅助表".into()),
        Some(format!("输入: {datetime}")),
        Some(body),
        // Compatibility text: the reference contract names its TypeScript
        // engine here. Internally this port uses the same-source tyme4rs.
        Some("来源: taibu-core/data/shensha 静态神煞表；按当前 tyme4ts 四柱命中计算。".into()),
        Some(
            "边界: 神煞为辅助参考，不可单独断事；必须回到十神、五行、月令、组合和流运同看。".into(),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_table_escapes_cell_content() {
        let rendered = table(&["A"], vec![vec!["x|y\nz".into()]]);
        assert!(rendered.contains("x\\|y<br>z"));
    }
}

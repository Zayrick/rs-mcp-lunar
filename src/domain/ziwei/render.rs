use std::collections::HashSet;

use iztro::core::labels::zh_cn;
use iztro::{
    DecorativeStarFamily, EarthlyBranch, Mutagen, PalaceName, Scope, StarName,
    StaticChartViewSnapshot, StaticPalaceRole, StaticPalaceView, StaticTypedStarView,
    TemporalContext, TemporalLayer, birth_year_star_mutagen, soul_master,
};

use super::engine::{
    ChartContext, HoroscopeContext, ZiweiProfile, ZiweiScope, ZiweiTopic, scope_solar_range,
};
use super::locale::Language;

const MUTAGENS: [Mutagen; 4] = [Mutagen::Lu, Mutagen::Quan, Mutagen::Ke, Mutagen::Ji];
// These are the append orders in iztro@2.5.8 `majorStar`, `minorStar`, and
// `adjectiveStar`. iztro-rs deliberately sorts its renderer-neutral snapshots
// by enum identity, so the compatibility renderer restores upstream array
// order without changing any calculated placement fact.
const MAJOR_STAR_ORDER: [StarName; 14] = [
    StarName::ZiWei,
    StarName::TianJi,
    StarName::TaiYang,
    StarName::WuQu,
    StarName::TianTong,
    StarName::LianZhen,
    StarName::TianFu,
    StarName::TaiYin,
    StarName::TanLang,
    StarName::JuMen,
    StarName::TianXiang,
    StarName::TianLiang,
    StarName::QiSha,
    StarName::PoJun,
];
const MINOR_STAR_ORDER: [StarName; 14] = [
    StarName::ZuoFu,
    StarName::YouBi,
    StarName::WenChang,
    StarName::WenQu,
    StarName::TianKui,
    StarName::TianYue,
    StarName::LuCun,
    StarName::TianMa,
    StarName::DiKong,
    StarName::DiJie,
    StarName::HuoXing,
    StarName::LingXing,
    StarName::QingYang,
    StarName::TuoLuo,
];
const ADJECTIVE_STAR_ORDER: [StarName; 42] = [
    StarName::HongLuan,
    StarName::TianXi,
    StarName::TianYao,
    StarName::XianChi,
    StarName::JieShen,
    StarName::SanTai,
    StarName::BaZuo,
    StarName::EnGuang,
    StarName::TianGui,
    StarName::LongChi,
    StarName::FengGe,
    StarName::TianCai,
    StarName::TianShou,
    StarName::TaiFu,
    StarName::FengGao,
    StarName::TianWu,
    StarName::HuaGai,
    StarName::TianGuan,
    StarName::TianFuAdj,
    StarName::TianChu,
    StarName::TianYueAdj,
    StarName::TianDe,
    StarName::YueDe,
    StarName::TianKong,
    StarName::XunKong,
    StarName::JieLu,
    StarName::KongWang,
    StarName::LongDeAdj,
    StarName::JieKong,
    StarName::JieShaAdj,
    StarName::DaHaoAdj,
    StarName::GuChen,
    StarName::GuaSu,
    StarName::FeiLian,
    StarName::PoSui,
    StarName::TianXing,
    StarName::YinSha,
    StarName::TianKu,
    StarName::TianXu,
    StarName::TianShi,
    StarName::TianShang,
    StarName::NianJie,
];
const REFERENCE_BRANCH_ORDER: [EarthlyBranch; 12] = [
    EarthlyBranch::Yin,
    EarthlyBranch::Mao,
    EarthlyBranch::Chen,
    EarthlyBranch::Si,
    EarthlyBranch::Wu,
    EarthlyBranch::Wei,
    EarthlyBranch::Shen,
    EarthlyBranch::You,
    EarthlyBranch::Xu,
    EarthlyBranch::Hai,
    EarthlyBranch::Zi,
    EarthlyBranch::Chou,
];

struct OverviewRow {
    label: String,
    branch: String,
    solar_range: String,
    origin_palace: String,
    mutagens: String,
    star_hint: String,
}

struct FlyingTransform<'a> {
    mutagen: Mutagen,
    target: Option<&'a StaticPalaceView>,
    is_self: bool,
}

pub(crate) fn chart(context: &ChartContext) -> String {
    let center = &context.view.center;
    let language = context.language;
    // Upstream zhongzhou selects 命主 from the birth-year branch, whereas
    // quanshu selects it from the life-palace branch. iztro-rs exposes the
    // shared lookup table, so only the selector needs adapting here.
    let soul = if context.profile == ZiweiProfile::FeixingSihua {
        Some(soul_master(center.birth_year_branch))
    } else {
        center.soul_master
    };
    let palace_rows = ordered_palaces(&context.view)
        .into_iter()
        .enumerate()
        .map(|(index, palace)| {
            vec![
                (index + 1).to_string(),
                language.palace(palace.name).to_owned(),
                stem_branch(palace, language),
                palace_flags(context, palace),
                yes_no(is_empty(palace)).to_owned(),
                star_list(&palace.major_stars, language),
                star_list(&palace.minor_stars, language),
                star_list(&palace.adjective_stars, language),
                decorative_sequence(palace, language),
                decadal_text(palace, language),
                ages_text(palace),
            ]
        })
        .collect();

    let transform_rows: Vec<Vec<String>> = ordered_palaces(&context.view)
        .into_iter()
        .flat_map(|palace| {
            palace
                .major_stars
                .iter()
                .filter_map(|star| {
                    star.mutagen.map(|mutagen| {
                        vec![
                            language.palace(palace.name).to_owned(),
                            language.star(star.name).to_owned(),
                            language.mutagen(mutagen).to_owned(),
                        ]
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let transforms = if transform_rows.is_empty() {
        "四化: 未见主星四化。".to_owned()
    } else {
        md_table(&["宫位", "星曜", "四化"], transform_rows)
    };

    join_sections([
        Some("紫微斗数本命全盘".to_owned()),
        Some(format!(
            "输入: {}\n历法: {}\n流派: {} ({})\n时辰索引: {}\n公历: {}\n农历: {}\n干支: {}",
            context.input_datetime,
            context.calendar.label(),
            context.profile.label(),
            context.profile.as_str(),
            context.time_index,
            context.solar_label,
            context.lunar_label,
            four_pillars_text(context),
        )),
        Some(md_table(
            &["命主", "身主", "五行局", "生肖", "星座", "命宫地支", "身宫地支"],
            vec![vec![
                soul.map_or_else(dash, |star| language.star(star).to_owned()),
                center
                    .body_master
                    .map_or_else(dash, |star| language.star(star).to_owned()),
                center
                    .five_element_bureau
                    .map_or_else(dash, |bureau| language.five_element_bureau(bureau).to_owned()),
                language.zodiac_animal(center.birth_year_branch).to_owned(),
                context
                    .western_zodiac
                    .map_or_else(dash, |sign| language.western_zodiac(sign).to_owned()),
                center
                    .life_palace_branch
                    .map_or_else(dash, |branch| language.earthly_branch(branch).to_owned()),
                center
                    .body_palace_branch
                    .map_or_else(dash, |branch| language.earthly_branch(branch).to_owned()),
            ]],
        )),
        Some(join_sections([
            Some("十二宫星曜".to_owned()),
            Some(md_table(
                &[
                    "序",
                    "宫位",
                    "干支",
                    "标记",
                    "空宫",
                    "主星",
                    "辅星",
                    "杂耀",
                    "长生/博士/将前/岁前",
                    "大限",
                    "小限年龄",
                ],
                palace_rows,
            )),
        ])),
        Some(join_sections([
            Some("生年四化".to_owned()),
            Some(transforms),
        ])),
        Some("边界: 本工具只给本命全盘。看单宫三方四正、夹宫、空宫借星与飞化自化，请调用 ziwei_palace_detail；看运限请调用 ziwei_horoscope_overview。".to_owned()),
    ])
}

pub(crate) fn palace_detail(context: &ChartContext, palace_input: &str) -> Result<String, String> {
    let language = context.language;
    let palace_name =
        parse_palace_name(palace_input).ok_or_else(|| format!("Unknown palace: {palace_input}"))?;
    let palace = palace_by_name(&context.view, palace_name)
        .ok_or_else(|| format!("Unknown palace: {palace_input}"))?;
    let relationships = natal_surrounded(&context.view, palace.branch);
    let adjacent = [
        ("前一宫", palace.branch.offset(-1)),
        ("后一宫", palace.branch.offset(1)),
    ];
    let flying = flying_transforms(context, palace);

    Ok(join_sections([
        Some("紫微斗数单宫详盘".to_owned()),
        Some(format!(
            "输入: {}\n历法: {}\n流派: {} ({})\n公历: {}\n农历: {}",
            context.input_datetime,
            context.calendar.label(),
            context.profile.label(),
            context.profile.as_str(),
            context.solar_label,
            context.lunar_label,
        )),
        Some(format!(
            "宫位: {}({})\n标记: {}\n空宫: {}",
            language.palace(palace.name),
            stem_branch(palace, language),
            palace_flags(context, palace),
            yes_no(is_empty(palace)),
        )),
        Some(md_table(
            &["项目", "主星", "辅星", "杂曜", "长生/博士/将前/岁前", "大限", "小限年龄"],
            vec![vec![
                format!(
                    "{}({})",
                    language.palace(palace.name),
                    stem_branch(palace, language)
                ),
                star_list(&palace.major_stars, language),
                star_list(&palace.minor_stars, language),
                star_list(&palace.adjective_stars, language),
                decorative_sequence(palace, language),
                decadal_text(palace, language),
                ages_text(palace),
            ]],
        )),
        Some(md_table(
            &["关系", "宫位", "干支", "主星", "辅星"],
            relationships
                .iter()
                .map(|(relation, item)| {
                    vec![
                        (*relation).to_owned(),
                        language.palace(item.name).to_owned(),
                        stem_branch(item, language),
                        star_list(&item.major_stars, language),
                        star_list(&item.minor_stars, language),
                    ]
                })
                .collect(),
        )),
        Some(md_table(
            &["夹宫", "宫位", "干支", "主星"],
            adjacent
                .iter()
                .filter_map(|(relation, branch)| {
                    palace_by_branch(&context.view, *branch).map(|item| {
                        vec![
                            (*relation).to_owned(),
                            language.palace(item.name).to_owned(),
                            stem_branch(item, language),
                            star_list(&item.major_stars, language),
                        ]
                    })
                })
                .collect(),
        )),
        is_empty(palace).then(|| {
            let opposite = palace_by_branch(&context.view, palace.branch.offset(6));
            opposite.map_or_else(
                || "空宫借星参考: 对宫 - 主星 -".to_owned(),
                |item| {
                    format!(
                        "空宫借星参考: 对宫 {} 主星 {}",
                        language.palace(item.name),
                        star_list(&item.major_stars, language)
                    )
                },
            )
        }),
        Some(md_table(
            &["宫干四化", "飞入宫位", "自化"],
            flying
                .iter()
                .map(|item| {
                    vec![
                        zh_cn::mutagen_zh(item.mutagen).to_owned(),
                        item.target.map_or_else(dash, |target| {
                            format!(
                                "{}({})",
                                language.palace(target.name),
                                stem_branch(target, language)
                            )
                        }),
                        yes_no(item.is_self).to_owned(),
                    ]
                })
                .collect(),
        )),
        Some("边界: 此工具只给单宫证据，不直接给最终吉凶断语。需要运限叠盘时调用 ziwei_scope_detail 或 ziwei_topic_context。".to_owned()),
    ]))
}

pub(crate) fn horoscope_overview(context: &HoroscopeContext) -> Result<String, String> {
    let rows = overview_rows(context)?;
    Ok(join_sections([
        Some("紫微运限总览".to_owned()),
        Some(format!(
            "出生: {} ({})\n目标: {}\n目标公历: {}\n目标农历: {}\n流派: {} ({})\n目标时辰索引: {}",
            context.birth_datetime,
            context.natal.calendar.label(),
            context.target_datetime,
            context.target_solar_label,
            context.target_lunar_label,
            context.natal.profile.label(),
            context.natal.profile.as_str(),
            context.target_time_index,
        )),
        Some(overview_table(rows)),
        Some("边界: 这个工具只做导航总览，不展开十二宫。下一步必须调用 ziwei_scope_detail 展开单层运限，或调用 ziwei_topic_context 做专题取证。".to_owned()),
    ]))
}

pub(crate) fn scope_detail(
    context: &HoroscopeContext,
    scope: ZiweiScope,
    focus_input: &str,
) -> Result<String, String> {
    let language = context.natal.language;
    let runtime = context.runtime()?;
    let layer = scope_layer(context, scope)?;
    let origin = runtime
        .palace(scope.iztro(), PalaceName::Life)
        .map_err(|error| error.to_string())?;
    let origin_view = palace_by_branch(&context.natal.view, origin.branch())
        .ok_or_else(|| "Missing origin palace.".to_owned())?;
    let nominal_age = match layer.context() {
        TemporalContext::Age { nominal_age, .. } => Some(*nominal_age),
        _ => None,
    };
    let scope_summary = md_table(
        &[
            "层级",
            "干支",
            "公历实际对应范围",
            "所在原盘宫",
            "四化",
            "虚岁",
        ],
        vec![vec![
            scope.label().to_owned(),
            language.stem_branch(layer.context().stem_branch()),
            scope_solar_range(scope, &context.target_solar_time)?,
            format!(
                "{}({})",
                language.palace(origin_view.name),
                stem_branch(origin_view, language)
            ),
            layer_mutagen_stars(layer, language),
            nominal_age.map_or_else(dash, |age| format!("{age}虚岁")),
        ]],
    );
    let palace_rows = ordered_palaces(&context.natal.view)
        .into_iter()
        .enumerate()
        .map(|(index, palace)| {
            let runtime_palace = layer
                .palace_layout()
                .and_then(|layout| layout.name_for_branch(palace.branch))
                .map(|name| language.palace(name))
                .unwrap_or("-");
            vec![
                (index + 1).to_string(),
                runtime_palace.to_owned(),
                language.palace(palace.name).to_owned(),
                stem_branch(palace, language),
                palace_flags(&context.natal, palace),
                yes_no(is_empty(palace)).to_owned(),
                star_list(&palace.major_stars, language),
                star_list(&palace.minor_stars, language),
                star_list(&palace.adjective_stars, language),
                layer_stars_at(layer, palace.branch, language),
            ]
        })
        .collect();

    let focus = if scope == ZiweiScope::Age {
        runtime
            .age_palace()
            .map_err(|error| error.to_string())?
            .branch()
    } else {
        let focus_name = parse_palace_name(focus_input)
            .ok_or_else(|| format!("Unknown focusPalace: {focus_input}"))?;
        runtime
            .palace(scope.iztro(), focus_name)
            .map_err(|_| format!("Unknown focusPalace: {focus_input}"))?
            .branch()
    };
    let focus_section = if scope == ZiweiScope::Age {
        format!(
            "小限宫位: {}",
            palace_by_branch(&context.natal.view, focus)
                .map(|palace| language.palace(palace.name))
                .unwrap_or("-")
        )
    } else {
        md_table(
            &["关系", "原盘宫位", "干支", "主星", "辅星"],
            natal_surrounded(&context.natal.view, focus)
                .iter()
                .map(|(relation, palace)| {
                    vec![
                        (*relation).to_owned(),
                        language.palace(palace.name).to_owned(),
                        stem_branch(palace, language),
                        star_list(&palace.major_stars, language),
                        star_list(&palace.minor_stars, language),
                    ]
                })
                .collect(),
        )
    };

    Ok(join_sections([
        Some("紫微单层运限详盘".to_owned()),
        Some(format!(
            "层级: {}\n重点宫位: {}\n出生: {}\n目标: {}\n目标公历: {}\n目标农历: {}\n流派: {} ({})",
            scope.label(),
            focus_input,
            context.birth_datetime,
            context.target_datetime,
            context.target_solar_label,
            context.target_lunar_label,
            context.natal.profile.label(),
            context.natal.profile.as_str(),
        )),
        Some(scope_summary),
        Some(md_table(
            &[
                "序",
                "运限宫位",
                "原盘宫位",
                "原盘干支",
                "标记",
                "空宫",
                "原盘主星",
                "原盘辅星",
                "原盘杂曜",
                "流耀",
            ],
            palace_rows,
        )),
        Some(focus_section),
        Some("边界: 此工具只展开一个运限层级。专题整合请调用 ziwei_topic_context。".to_owned()),
    ]))
}

pub(crate) fn topic_context(
    context: &HoroscopeContext,
    topic: ZiweiTopic,
) -> Result<String, String> {
    let language = context.natal.language;
    let runtime = context.runtime()?;
    let yearly_layer = scope_layer(context, ZiweiScope::Yearly)?;
    let topic_palaces = topic_palaces(topic);
    let overview = overview_rows(context)?;

    let evidence_rows = topic_palaces
        .iter()
        .map(|name| {
            let natal = palace_by_name(&context.natal.view, *name)
                .ok_or_else(|| format!("Unknown topic palace: {}", zh_cn::palace_name_zh(*name)))?;
            let yearly = runtime
                .palace(Scope::Yearly, *name)
                .map_err(|error| error.to_string())?;
            let yearly_palace = palace_by_branch(&context.natal.view, yearly.branch());
            let mutagen_hits = MUTAGENS
                .iter()
                .filter_map(|mutagen| {
                    runtime
                        .has_horoscope_mutagen(Scope::Yearly, *name, *mutagen)
                        .ok()
                        .filter(|hit| *hit)
                        .map(|_| zh_cn::mutagen_zh(*mutagen))
                })
                .collect::<Vec<_>>()
                .join("、");
            Ok(vec![
                zh_cn::palace_name_zh(*name).to_owned(),
                stem_branch(natal, language),
                star_list(&natal.major_stars, language),
                star_list(&natal.minor_stars, language),
                natal_mutagen_stars(natal, language),
                yearly_palace.map_or_else(dash, |palace| {
                    format!(
                        "{}({})",
                        language.palace(palace.name),
                        stem_branch(palace, language)
                    )
                }),
                layer_stars_at(yearly_layer, natal.branch, language),
                value_or_dash(&mutagen_hits),
            ])
        })
        .collect::<Result<Vec<_>, String>>()?;

    let detail_rows = topic_palaces
        .iter()
        .map(|name| {
            let natal = palace_by_name(&context.natal.view, *name)
                .ok_or_else(|| format!("Unknown topic palace: {}", zh_cn::palace_name_zh(*name)))?;
            let surrounded = natal_surrounded(&context.natal.view, natal.branch)
                .into_iter()
                .map(|(relation, palace)| {
                    format!(
                        "{relation}:{}({})",
                        language.palace(palace.name),
                        star_list(&palace.major_stars, language)
                    )
                })
                .collect::<Vec<_>>()
                .join("；");
            let flying = flying_transforms(&context.natal, natal)
                .into_iter()
                .map(|item| {
                    let target = item.target.map_or_else(dash, |palace| {
                        format!(
                            "{}({})",
                            language.palace(palace.name),
                            stem_branch(palace, language)
                        )
                    });
                    format!(
                        "{}->{target}{}",
                        zh_cn::mutagen_zh(item.mutagen),
                        if item.is_self { "[自化]" } else { "" }
                    )
                })
                .collect::<Vec<_>>()
                .join("；");
            let palace_label = zh_cn::palace_name_zh(*name).to_owned();
            Ok(vec![
                palace_label.clone(),
                surrounded,
                flying,
                format!(
                    "ziwei_palace_detail(palace={palace_label})；ziwei_scope_detail(scope=yearly, focusPalace={palace_label})"
                ),
            ])
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(join_sections([
        Some("紫微专题取证".to_owned()),
        Some(format!(
            "专题: {}\n出生: {}\n目标: {}\n相关宫位: {}\n流派: {} ({})\n历法: {}\n目标公历: {}\n目标农历: {}\n目标时辰索引: {}",
            topic.as_str(),
            context.birth_datetime,
            context.target_datetime,
            topic_palaces
                .iter()
                .map(|name| zh_cn::palace_name_zh(*name))
                .collect::<Vec<_>>()
                .join("、"),
            context.natal.profile.label(),
            context.natal.profile.as_str(),
            context.natal.calendar.label(),
            context.target_solar_label,
            context.target_lunar_label,
            context.target_time_index,
        )),
        Some(overview_table(overview)),
        Some(md_table(
            &[
                "宫位",
                "本命干支",
                "本命主星",
                "本命辅星",
                "本命四化星",
                "流年宫位",
                "流年流耀",
                "流年四化命中",
            ],
            evidence_rows,
        )),
        Some(md_table(
            &["宫位", "本命三方四正", "本命飞化", "建议调用"],
            detail_rows,
        )),
        Some("边界: 专题取证只返回索引级证据，不输出最终断语；单宫细节调用 ziwei_palace_detail，运限叠盘调用 ziwei_scope_detail。".to_owned()),
    ]))
}

fn overview_rows(context: &HoroscopeContext) -> Result<Vec<OverviewRow>, String> {
    let runtime = context.runtime()?;
    let language = context.natal.language;
    ZiweiScope::ALL
        .iter()
        .map(|scope| {
            let layer = scope_layer(context, *scope)?;
            let origin = runtime
                .palace(scope.iztro(), PalaceName::Life)
                .map_err(|error| error.to_string())?;
            let origin_palace = palace_by_branch(&context.natal.view, origin.branch())
                .ok_or_else(|| "Missing origin palace.".to_owned())?;
            let label = match layer.context() {
                TemporalContext::Age { nominal_age, .. } => {
                    format!("{}({nominal_age}虚岁)", scope.label())
                }
                _ => scope.label().to_owned(),
            };
            let star_count = layer_star_palace_count(layer);
            Ok(OverviewRow {
                label,
                branch: language.stem_branch(layer.context().stem_branch()),
                solar_range: scope_solar_range(*scope, &context.target_solar_time)?,
                origin_palace: format!(
                    "{}({})",
                    language.palace(origin_palace.name),
                    stem_branch(origin_palace, language)
                ),
                mutagens: layer_mutagen_stars(layer, language),
                star_hint: if star_count == 0 {
                    dash()
                } else {
                    format!("有流耀宫位 {star_count}/12")
                },
            })
        })
        .collect()
}

fn overview_table(rows: Vec<OverviewRow>) -> String {
    md_table(
        &[
            "层级",
            "干支",
            "公历实际对应范围",
            "所在原盘宫",
            "四化",
            "流耀提示",
        ],
        rows.into_iter()
            .map(|row| {
                vec![
                    row.label,
                    row.branch,
                    row.solar_range,
                    row.origin_palace,
                    row.mutagens,
                    row.star_hint,
                ]
            })
            .collect(),
    )
}

fn scope_layer(context: &HoroscopeContext, scope: ZiweiScope) -> Result<&TemporalLayer, String> {
    context
        .horoscope
        .layers_in_scope(scope.iztro())
        .next()
        .ok_or_else(|| format!("Missing {} horoscope layer.", scope.label()))
}

fn layer_mutagen_stars(layer: &TemporalLayer, language: Language) -> String {
    let names = MUTAGENS
        .iter()
        .filter_map(|mutagen| {
            layer
                .activations()
                .iter()
                .find(|activation| activation.mutagen() == *mutagen)
                .map(|activation| language.star(activation.target_star()))
        })
        .collect::<Vec<_>>()
        .join("、");
    value_or_dash(&names)
}

fn layer_stars_at(layer: &TemporalLayer, branch: EarthlyBranch, language: Language) -> String {
    let names = layer
        .placements()
        .iter()
        .filter(|placement| placement.branch() == branch)
        .map(|placement| language.star(placement.placement().name()))
        .collect::<Vec<_>>()
        .join("、");
    value_or_dash(&names)
}

fn layer_star_palace_count(layer: &TemporalLayer) -> usize {
    layer
        .placements()
        .iter()
        .map(|placement| placement.branch())
        .collect::<HashSet<_>>()
        .len()
}

fn natal_mutagen_stars(palace: &StaticPalaceView, language: Language) -> String {
    let mut stars = palace
        .major_stars
        .iter()
        .chain(&palace.minor_stars)
        .chain(&palace.adjective_stars)
        .filter(|star| star.mutagen.is_some())
        .collect::<Vec<_>>();
    stars.sort_by_key(|star| upstream_star_rank(star.name));
    let stars = stars
        .into_iter()
        .map(|star| star_text(star, language))
        .collect::<Vec<_>>()
        .join("、");
    value_or_dash(&stars)
}

fn flying_transforms<'a>(
    context: &'a ChartContext,
    source: &StaticPalaceView,
) -> Vec<FlyingTransform<'a>> {
    MUTAGENS
        .iter()
        .map(|mutagen| {
            let target = context.chart.stars().into_iter().find_map(|fact| {
                (birth_year_star_mutagen(source.stem, fact.placement().name()) == Some(*mutagen))
                    .then(|| palace_by_branch(&context.view, fact.palace().branch()))
                    .flatten()
            });
            FlyingTransform {
                mutagen: *mutagen,
                is_self: target.is_some_and(|palace| palace.branch == source.branch),
                target,
            }
        })
        .collect()
}

fn ordered_palaces(view: &StaticChartViewSnapshot) -> Vec<&StaticPalaceView> {
    REFERENCE_BRANCH_ORDER
        .iter()
        .filter_map(|branch| palace_by_branch(view, *branch))
        .collect()
}

fn palace_by_branch(
    view: &StaticChartViewSnapshot,
    branch: EarthlyBranch,
) -> Option<&StaticPalaceView> {
    view.palaces.iter().find(|palace| palace.branch == branch)
}

fn palace_by_name(view: &StaticChartViewSnapshot, name: PalaceName) -> Option<&StaticPalaceView> {
    view.palaces.iter().find(|palace| palace.name == name)
}

fn natal_surrounded(
    view: &StaticChartViewSnapshot,
    branch: EarthlyBranch,
) -> Vec<(&'static str, &StaticPalaceView)> {
    [
        ("本宫", branch),
        ("对宫", branch.offset(6)),
        ("财帛位", branch.offset(8)),
        ("官禄位", branch.offset(4)),
    ]
    .into_iter()
    .filter_map(|(relation, branch)| {
        palace_by_branch(view, branch).map(|palace| (relation, palace))
    })
    .collect()
}

fn parse_palace_name(value: &str) -> Option<PalaceName> {
    match value.trim() {
        "命" | "命宫" => Some(PalaceName::Life),
        "兄弟" | "兄弟宫" => Some(PalaceName::Siblings),
        "夫妻" | "夫妻宫" => Some(PalaceName::Spouse),
        "子女" | "子女宫" => Some(PalaceName::Children),
        "财帛" | "财帛宫" => Some(PalaceName::Wealth),
        "疾厄" | "疾厄宫" => Some(PalaceName::Health),
        "迁移" | "迁移宫" => Some(PalaceName::Migration),
        "仆役" | "仆役宫" | "交友" | "交友宫" => Some(PalaceName::Friends),
        "官禄" | "官禄宫" | "事业" | "事业宫" => Some(PalaceName::Career),
        "田宅" | "田宅宫" => Some(PalaceName::Property),
        "福德" | "福德宫" => Some(PalaceName::Spirit),
        "父母" | "父母宫" => Some(PalaceName::Parents),
        _ => None,
    }
}

fn topic_palaces(topic: ZiweiTopic) -> &'static [PalaceName] {
    match topic {
        ZiweiTopic::SelfAnalysis => &[PalaceName::Life, PalaceName::Migration, PalaceName::Spirit],
        ZiweiTopic::Career => &[
            PalaceName::Career,
            PalaceName::Migration,
            PalaceName::Wealth,
            PalaceName::Spirit,
        ],
        ZiweiTopic::Wealth => &[
            PalaceName::Wealth,
            PalaceName::Property,
            PalaceName::Career,
            PalaceName::Siblings,
        ],
        ZiweiTopic::Relationship => &[
            PalaceName::Spouse,
            PalaceName::Children,
            PalaceName::Spirit,
            PalaceName::Life,
        ],
        ZiweiTopic::Health => &[
            PalaceName::Health,
            PalaceName::Spirit,
            PalaceName::Life,
            PalaceName::Parents,
        ],
        ZiweiTopic::Family => &[
            PalaceName::Property,
            PalaceName::Parents,
            PalaceName::Siblings,
            PalaceName::Children,
            PalaceName::Spouse,
        ],
    }
}

fn palace_flags(context: &ChartContext, palace: &StaticPalaceView) -> String {
    let mut flags = Vec::new();
    if palace.roles.contains(&StaticPalaceRole::BodyPalace) {
        flags.push("身宫");
    }
    // Upstream iztro defines 来因宫 as the non-子丑 palace whose palace stem
    // equals the birth-year stem. iztro-rs 0.9 does not export this convenience
    // flag, so the adapter derives only that documented identity predicate.
    if !matches!(palace.branch, EarthlyBranch::Zi | EarthlyBranch::Chou)
        && palace.stem == context.view.center.birth_year_stem
    {
        flags.push("来因宫");
    }
    if flags.is_empty() {
        dash()
    } else {
        flags.join("、")
    }
}

fn is_empty(palace: &StaticPalaceView) -> bool {
    palace.major_stars.is_empty()
}

fn four_pillars_text(context: &ChartContext) -> String {
    let romanized = matches!(context.language, Language::EnUs | Language::ViVn);
    let pillar_separator = if romanized { " - " } else { " " };
    context
        .chinese_date
        .iter()
        .map(|pillar| {
            if romanized {
                format!(
                    "{} {}",
                    context.language.heavenly_stem(pillar.stem()),
                    context.language.earthly_branch(pillar.branch())
                )
            } else {
                context.language.stem_branch(*pillar)
            }
        })
        .collect::<Vec<_>>()
        .join(pillar_separator)
}

fn stem_branch(palace: &StaticPalaceView, language: Language) -> String {
    format!(
        "{}{}",
        language.heavenly_stem(palace.stem),
        language.earthly_branch(palace.branch)
    )
}

fn star_text(star: &StaticTypedStarView, language: Language) -> String {
    let brightness = language.brightness(star.brightness);
    format!(
        "{}{}{}",
        language.star(star.name),
        if brightness.is_empty() {
            String::new()
        } else {
            format!("({brightness})")
        },
        star.mutagen.map_or_else(String::new, |mutagen| format!(
            "[{}]",
            language.mutagen(mutagen)
        )),
    )
}

fn star_list(stars: &[StaticTypedStarView], language: Language) -> String {
    if stars.is_empty() {
        dash()
    } else {
        let mut ordered = stars.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|star| upstream_star_rank(star.name));
        ordered
            .into_iter()
            .map(|star| star_text(star, language))
            .collect::<Vec<_>>()
            .join("、")
    }
}

fn upstream_star_rank(name: StarName) -> usize {
    MAJOR_STAR_ORDER
        .iter()
        .position(|candidate| *candidate == name)
        .or_else(|| {
            MINOR_STAR_ORDER
                .iter()
                .position(|candidate| *candidate == name)
                .map(|index| MAJOR_STAR_ORDER.len() + index)
        })
        .or_else(|| {
            ADJECTIVE_STAR_ORDER
                .iter()
                .position(|candidate| *candidate == name)
                .map(|index| MAJOR_STAR_ORDER.len() + MINOR_STAR_ORDER.len() + index)
        })
        .unwrap_or(usize::MAX)
}

fn decorative_sequence(palace: &StaticPalaceView, language: Language) -> String {
    [
        DecorativeStarFamily::Changsheng12,
        DecorativeStarFamily::Boshi12,
        DecorativeStarFamily::Jiangqian12,
        DecorativeStarFamily::Suiqian12,
    ]
    .iter()
    .map(|family| {
        palace
            .decorative_stars
            .iter()
            .find(|star| star.family == *family)
            .map(|star| language.decorative_star(star.name).to_owned())
            .unwrap_or_else(dash)
    })
    .collect::<Vec<_>>()
    .join("/")
}

fn decadal_text(palace: &StaticPalaceView, language: Language) -> String {
    palace
        .limit
        .decadal_age_range_zh
        .as_ref()
        .map_or_else(dash, |range| {
            format!("{range}岁 {}", stem_branch(palace, language))
        })
}

fn ages_text(palace: &StaticPalaceView) -> String {
    if palace.limit.small_limit_ages_zh.is_empty() {
        dash()
    } else {
        palace.limit.small_limit_ages_zh.join("、")
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "是" } else { "否" }
}

fn dash() -> String {
    "-".to_owned()
}

fn value_or_dash(value: &str) -> String {
    if value.is_empty() {
        dash()
    } else {
        value.to_owned()
    }
}

fn md_value(value: &str) -> String {
    value.replace(['\r', '\n'], "<br>").replace('|', "\\|")
}

fn md_table(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let header = format!(
        "| {} |",
        headers
            .iter()
            .map(|value| md_value(value))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let separator = format!(
        "| {} |",
        headers
            .iter()
            .map(|_| "---")
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let body = rows
        .iter()
        .map(|row| {
            format!(
                "| {} |",
                row.iter()
                    .map(|value| md_value(value))
                    .collect::<Vec<_>>()
                    .join(" | ")
            )
        })
        .collect::<Vec<_>>();
    std::iter::once(header)
        .chain(std::iter::once(separator))
        .chain(body)
        .collect::<Vec<_>>()
        .join("\n")
}

fn join_sections<const N: usize>(sections: [Option<String>; N]) -> String {
    sections
        .into_iter()
        .flatten()
        .filter(|section| !section.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::domain::ziwei::engine::{
        build_chart, build_horoscope, common_chart_args, horoscope_args,
    };

    fn object(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn natal_chart_has_contract_title_and_tables() {
        let args = object(json!({"datetime":"2024-01-15 08:30","gender":"男"}));
        let output = chart(&build_chart(common_chart_args(&args, "datetime").unwrap()).unwrap());
        assert!(output.starts_with("紫微斗数本命全盘"));
        assert!(output.contains("| 序 | 宫位 | 干支 |"));
        assert!(output.contains("边界: 本工具只给本命全盘"));
    }

    #[test]
    fn overview_has_exact_hour_range() {
        let args = object(json!({
            "birthDatetime":"2024-01-15 08:30",
            "gender":"男",
            "targetDatetime":"2026-06-12 18:00"
        }));
        let context = build_horoscope(horoscope_args(&args).unwrap()).unwrap();
        let output = horoscope_overview(&context).unwrap();
        assert!(output.contains("紫微运限总览"));
        assert!(output.contains("2026-06-12 17:00:00 至 2026-06-12 18:59:59"));
    }
}

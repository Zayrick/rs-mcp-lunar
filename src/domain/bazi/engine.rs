use std::cmp::Ordering;

use tyme4rs::tyme::eightchar::{ChildLimit, EightChar};
use tyme4rs::tyme::enums::{Gender, HideHeavenStemType};
use tyme4rs::tyme::sixtycycle::{
    EarthBranch, HeavenStem, SixtyCycle, SixtyCycleDay, SixtyCycleHour, SixtyCycleMonth,
    SixtyCycleYear,
};
use tyme4rs::tyme::solar::{SolarDay, SolarTerm, SolarTime};
use tyme4rs::tyme::{Culture, Tyme};

use super::{model::*, shensha};

const PILLAR_LABELS: [&str; 4] = ["年柱", "月柱", "日柱", "时柱"];
const ELEMENTS: [&str; 5] = ["木", "火", "土", "金", "水"];

#[derive(Debug, Clone, Copy)]
struct DateTimeParts {
    year: isize,
    month: usize,
    day: usize,
    hour: usize,
    minute: usize,
}

struct Context {
    parts: DateTimeParts,
    solar_time: SolarTime,
    solar_day: SolarDay,
    eight_char: EightChar,
    day_master: HeavenStem,
}

fn parse_datetime(input: &str) -> Result<DateTimeParts, String> {
    if input.len() != 16
        || input.as_bytes().get(4) != Some(&b'-')
        || input.as_bytes().get(7) != Some(&b'-')
        || input.as_bytes().get(10) != Some(&b' ')
        || input.as_bytes().get(13) != Some(&b':')
    {
        return Err("Invalid format. Use YYYY-MM-DD HH:MM".into());
    }
    let number = |range: std::ops::Range<usize>| {
        input[range]
            .parse::<usize>()
            .map_err(|_| "Invalid format. Use YYYY-MM-DD HH:MM".to_string())
    };
    Ok(DateTimeParts {
        year: number(0..4)? as isize,
        month: number(5..7)?,
        day: number(8..10)?,
        hour: number(11..13)?,
        minute: number(14..16)?,
    })
}

fn parse_date(input: &str) -> Result<(isize, usize, usize), String> {
    if input.len() != 10
        || input.as_bytes().get(4) != Some(&b'-')
        || input.as_bytes().get(7) != Some(&b'-')
    {
        return Err("invalid date".into());
    }
    let number = |range: std::ops::Range<usize>| {
        input[range]
            .parse::<usize>()
            .map_err(|_| "invalid date".to_string())
    };
    Ok((number(0..4)? as isize, number(5..7)?, number(8..10)?))
}

pub(crate) fn gender(input: &str) -> Result<(Gender, String), String> {
    match input.trim().to_lowercase().as_str() {
        "男" | "male" | "m" => Ok((Gender::MAN, "男".into())),
        "女" | "female" | "f" => Ok((Gender::WOMAN, "女".into())),
        _ => Err("Invalid gender. Use 男/女 or male/female.".into()),
    }
}

fn context(datetime: &str) -> Result<Context, String> {
    let parts = parse_datetime(datetime)?;
    validate_child_limit_year(parts.year)?;
    // SolarDay::validate reaches a panicking `SolarMonth::from_ym` convenience
    // constructor for invalid parent fields. Validate that parent fallibly so
    // malformed public input stays a normal tool error.
    tyme4rs::tyme::solar::SolarMonth::new(parts.year, parts.month)?;
    let solar_time = SolarTime::new(
        parts.year,
        parts.month,
        parts.day,
        parts.hour,
        parts.minute,
        0,
    )?;
    let solar_day = solar_time.get_solar_day();
    let eight_char = solar_time.get_lunar_hour().get_eight_char();
    let day_master = eight_char.get_day().get_heaven_stem();
    Ok(Context {
        parts,
        solar_time,
        solar_day,
        eight_char,
        day_master,
    })
}

/// `tyme4rs` exposes both fallible `new` and panicking `from_year` APIs. Range
/// rendering always needs the following sexagenary year, so validate both
/// years before entering any of the library's infallible convenience paths.
fn sixty_cycle_year_with_range(year: isize) -> Result<SixtyCycleYear, String> {
    let current = SixtyCycleYear::new(year)?;
    let following = year
        .checked_add(1)
        .ok_or_else(|| format!("illegal sixty cycle year: {year}"))?;
    SixtyCycleYear::new(following)?;
    if year == -1 {
        return Err("illegal solar year: 0".to_owned());
    }
    Ok(current)
}

fn validate_period_date_year(year: isize) -> Result<(), String> {
    // Day and double-hour renderers expand to inclusive start/end instants and
    // query annual calendar facts. At the tyme4rs boundary those paths can step
    // into year zero or 10000 through infallible convenience constructors.
    if !(2..=9_998).contains(&year) {
        return Err("period date exceeds tyme4rs safe range (year 2..=9998).".to_owned());
    }
    Ok(())
}

fn validate_child_limit_year(year: isize) -> Result<(), String> {
    // The default child-limit provider may advance the birth date by up to ten
    // years and internally uses panicking SolarMonth constructors. Keeping that
    // addition inside the documented SolarYear range prevents a poisoned global
    // provider mutex and a process abort.
    if year == 1 {
        return Err("illegal solar year: 0".to_owned());
    }
    if year > 9_988 {
        return Err("birth datetime exceeds tyme4rs safe range (year <= 9988).".to_owned());
    }
    Ok(())
}

fn hidden_stem_type(kind: HideHeavenStemType) -> (&'static str, f64) {
    match kind {
        HideHeavenStemType::MAIN => ("本气", 0.6),
        HideHeavenStemType::MIDDLE => ("中气", 0.3),
        HideHeavenStemType::RESIDUAL => ("余气", 0.1),
    }
}

fn pillar_detail(cycle: &SixtyCycle, day_master: &HeavenStem, is_day: bool) -> PillarInfo {
    let stem = cycle.get_heaven_stem();
    let branch = cycle.get_earth_branch();
    let hidden_stems = branch
        .get_hide_heaven_stems()
        .into_iter()
        .map(|hidden| {
            let hidden_stem = hidden.get_heaven_stem();
            let (qi_type, weight) = hidden_stem_type(hidden.get_type());
            HiddenStem {
                stem: hidden_stem.get_name(),
                qi_type,
                weight,
                ten_god: day_master.get_ten_star(hidden_stem).get_name(),
            }
        })
        .collect();
    let extras = cycle.get_extra_earth_branches();
    PillarInfo {
        pillar: cycle.get_name(),
        ten_god: if is_day {
            "日主".into()
        } else {
            day_master.get_ten_star(stem.clone()).get_name()
        },
        heaven_stem: stem.get_name(),
        earth_branch: branch.get_name(),
        hidden_stems,
        terrain: day_master.get_terrain(branch.clone()).get_name(),
        self_sitting: stem.get_terrain(branch).get_name(),
        kong_wang: extras
            .iter()
            .map(Culture::get_name)
            .collect::<Vec<_>>()
            .join(""),
        nayin: cycle.get_sound().get_name(),
        shensha: Vec::new(),
    }
}

fn cycles(eight_char: &EightChar) -> [SixtyCycle; 4] {
    [
        eight_char.get_year(),
        eight_char.get_month(),
        eight_char.get_day(),
        eight_char.get_hour(),
    ]
}

fn cycle_with_nayin(cycle: SixtyCycle) -> String {
    format!("{}({})", cycle.get_name(), cycle.get_sound().get_name())
}

/// `tyme4rs` 1.5.0 uses subtraction for the hour branch in `get_body_sign`,
/// while the same-source `tyme4ts` 1.5.2 used by the reference uses addition.
/// Keep this compatibility shim local until the Rust upstream is aligned.
fn reference_body_sign(eight_char: &EightChar) -> SixtyCycle {
    SixtyCycle::from_index(
        eight_char.get_year().get_heaven_stem().get_index() as isize * 12
            + (11
                + eight_char.get_month().get_earth_branch().get_index() as isize
                + eight_char.get_hour().get_earth_branch().get_index() as isize)
                % 12
            + 2,
    )
}

fn solar_text(time: SolarTime) -> String {
    format!(
        "{}-{:02}-{:02} {:02}:{:02}:{:02}",
        time.get_year(),
        time.get_month(),
        time.get_day(),
        time.get_hour(),
        time.get_minute(),
        time.get_second()
    )
}

fn solar_range(start: SolarTime, end: SolarTime) -> String {
    format!("{} 至 {}", solar_text(start), solar_text(end))
}

fn month_start(month: &SixtyCycleMonth) -> SolarTime {
    SolarTerm::from_index(
        month.get_sixty_cycle_year().get_year(),
        3 + month.get_index_in_year() as isize * 2,
    )
    .get_julian_day()
    .get_solar_time()
}

fn year_range(year: &SixtyCycleYear) -> String {
    let start = month_start(&year.get_first_month());
    let end = month_start(&year.next(1).get_first_month()).next(-1);
    solar_range(start, end)
}

fn month_range(month: &SixtyCycleMonth) -> String {
    solar_range(month_start(month), month_start(&month.next(1)).next(-1))
}

fn day_range(day: &SixtyCycleDay) -> String {
    let hours = day.get_hours();
    match (hours.first(), hours.last()) {
        (Some(first), Some(last)) => {
            solar_range(first.get_solar_time(), last.get_solar_time().next(7199))
        }
        _ => "-".into(),
    }
}

fn hour_range(hour: &SixtyCycleHour) -> String {
    let time = hour.get_solar_time();
    let start = if time.get_hour() == 0 {
        SolarTime::from_ymd_hms(time.get_year(), time.get_month(), time.get_day(), 0, 0, 0)
            .next(-3600)
    } else {
        let h = if time.get_hour() == 23 {
            23
        } else {
            time.get_hour().div_ceil(2) * 2 - 1
        };
        SolarTime::from_ymd_hms(time.get_year(), time.get_month(), time.get_day(), h, 0, 0)
    };
    solar_range(start, start.next(7199))
}

fn jie_pair(time: SolarTime) -> (TermInfo, TermInfo) {
    let mut current = time.get_term();
    if !current.is_jie() {
        current = current.next(-1);
    }
    let next = current.next(2);
    let term_info = |term: SolarTerm| {
        let t = term.get_julian_day().get_solar_time();
        TermInfo {
            name: term.get_name(),
            time: format!(
                "{}年{}月{}日 {:02}:{:02}:{:02}",
                t.get_year(),
                t.get_month(),
                t.get_day(),
                t.get_hour(),
                t.get_minute(),
                t.get_second()
            ),
        }
    };
    (term_info(current), term_info(next))
}

pub(crate) fn build_chart(datetime: &str, gender_input: &str) -> Result<ChartData, String> {
    let ctx = context(datetime)?;
    let (gender, gender_label) = gender(gender_input)?;
    let raw_cycles = cycles(&ctx.eight_char);
    let mut pillars =
        std::array::from_fn(|index| pillar_detail(&raw_cycles[index], &ctx.day_master, index == 2));
    for (pillar, hits) in pillars
        .iter_mut()
        .zip(shensha::pillar_hits(&ctx.eight_char))
    {
        pillar.shensha = hits;
    }
    let (current_jie, next_jie) = jie_pair(ctx.solar_time);
    let child = ChildLimit::from_solar_time(ctx.solar_time, gender);
    let end = child.get_end_time();
    let mut luck = vec![LuckRow {
        label: "童限".into(),
        cycle: None,
        ten_god: None,
        nayin: None,
        start_age: child.get_start_age() as isize,
        end_age: child.get_end_age() as isize,
        start_year: ctx.parts.year + child.get_start_age() as isize - 1,
        end_year: ctx.parts.year + child.get_end_age() as isize,
    }];
    let mut decade = child.get_start_decade_fortune();
    for index in 0..10 {
        let cycle = decade.get_sixty_cycle();
        luck.push(LuckRow {
            label: format!("第{}步大运", index + 1),
            cycle: Some(cycle.get_name()),
            ten_god: Some(
                ctx.day_master
                    .get_ten_star(cycle.get_heaven_stem())
                    .get_name(),
            ),
            nayin: Some(cycle.get_sound().get_name()),
            start_age: decade.get_start_age(),
            end_age: decade.get_end_age(),
            start_year: ctx.parts.year + decade.get_start_age() - 1,
            end_year: ctx.parts.year + decade.get_end_age() - 1,
        });
        decade = decade.next(1);
    }
    let lunar_day = ctx.solar_day.get_lunar_day();
    let day_extras = ctx.eight_char.get_day().get_extra_earth_branches();
    Ok(ChartData {
        input: datetime.into(),
        gender: gender_label,
        solar: ctx.solar_day.to_string(),
        lunar: format!("{} {}时", lunar_day, ctx.eight_char.get_hour().get_name()),
        four_pillar_text: raw_cycles
            .iter()
            .map(Culture::get_name)
            .collect::<Vec<_>>()
            .join(" "),
        day_master: ctx.day_master.get_name(),
        day_kong_wang: format!(
            "{}旬 {}空",
            ctx.eight_char.get_day().get_ten().get_name(),
            day_extras
                .iter()
                .map(Culture::get_name)
                .collect::<Vec<_>>()
                .join("、")
        ),
        commander: ctx.solar_day.get_hide_heaven_stem_day().get_name(),
        pillars,
        current_jie,
        next_jie,
        fetal_origin: cycle_with_nayin(ctx.eight_char.get_fetal_origin()),
        fetal_breath: cycle_with_nayin(ctx.eight_char.get_fetal_breath()),
        life_palace: cycle_with_nayin(ctx.eight_char.get_own_sign()),
        body_palace: cycle_with_nayin(reference_body_sign(&ctx.eight_char)),
        start_luck_description: format!(
            "{}年{}个月{}天{}时{}分后起运",
            child.get_year_count(),
            child.get_month_count(),
            child.get_day_count(),
            child.get_hour_count(),
            child.get_minute_count()
        ),
        start_luck_date: format!(
            "公历{}年{}月{}日 {:02}:{:02}:{:02}",
            end.get_year(),
            end.get_month(),
            end.get_day(),
            end.get_hour(),
            end.get_minute(),
            end.get_second()
        ),
        is_forward: child.is_forward(),
        luck,
    })
}

fn element_scores(eight_char: &EightChar) -> [f64; 5] {
    let mut scores = [0.0; 5];
    let add = |scores: &mut [f64; 5], name: String, value: f64| {
        if let Some(index) = ELEMENTS.iter().position(|candidate| *candidate == name) {
            scores[index] += value;
        }
    };
    for cycle in cycles(eight_char) {
        add(
            &mut scores,
            cycle.get_heaven_stem().get_element().get_name(),
            1.0,
        );
        let branch = cycle.get_earth_branch();
        add(&mut scores, branch.get_element().get_name(), 1.0);
        add(
            &mut scores,
            branch.get_hide_heaven_stem_main().get_element().get_name(),
            0.6,
        );
        if let Some(stem) = branch.get_hide_heaven_stem_middle() {
            add(&mut scores, stem.get_element().get_name(), 0.3);
        }
        if let Some(stem) = branch.get_hide_heaven_stem_residual() {
            add(&mut scores, stem.get_element().get_name(), 0.1);
        }
    }
    scores
}

fn branch_order(branch: &str) -> usize {
    [
        "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
    ]
    .iter()
    .position(|x| *x == branch)
    .unwrap_or(12)
}

fn pair_key(a: &str, b: &str) -> String {
    if branch_order(a) <= branch_order(b) {
        format!("{a}{b}")
    } else {
        format!("{b}{a}")
    }
}

fn branch_relations(a: EarthBranch, b: EarthBranch) -> Vec<String> {
    let pair = format!("{}{}", a.get_name(), b.get_name());
    let key = pair_key(&a.get_name(), &b.get_name());
    let mut result = Vec::new();
    if let Some(element) = a.combine(b.clone()) {
        result.push(format!("六合{}({pair})", element.get_name()));
    }
    if a.get_opposite() == b {
        result.push(format!("六冲({pair})"));
    }
    if a.get_harm() == b {
        result.push(format!("六害({pair})"));
    }
    if let Some(name) = match key.as_str() {
        "子卯" => Some("无礼刑"),
        "寅巳" | "寅申" | "巳申" => Some("无恩刑"),
        "丑未" | "丑戌" | "未戌" => Some("恃势刑"),
        _ => None,
    } {
        result.push(format!("{name}({pair})"));
    }
    if ["子酉", "丑辰", "寅亥", "卯午", "巳申", "未戌"].contains(&key.as_str()) {
        result.push(format!("破({pair})"));
    }
    result
}

fn relation_rows(eight_char: &EightChar) -> Vec<RelationRow> {
    let all = cycles(eight_char);
    let mut rows = Vec::new();
    let mut branch_map: Vec<(String, Vec<String>)> = Vec::new();
    for left in 0..4 {
        let left_cycle = &all[left];
        let branch_name = left_cycle.get_earth_branch().get_name();
        let label = format!("{}{}", PILLAR_LABELS[left], left_cycle.get_name());
        if let Some((_, labels)) = branch_map.iter_mut().find(|(name, _)| name == &branch_name) {
            labels.push(label);
        } else {
            branch_map.push((branch_name, vec![label]));
        }
        for right in (left + 1)..4 {
            let right_cycle = &all[right];
            if let Some(element) = left_cycle
                .get_heaven_stem()
                .combine(right_cycle.get_heaven_stem())
            {
                rows.push(RelationRow {
                    kind: "天干五合".into(),
                    relation: format!(
                        "{}{} 与 {}{}: {}{}合{}",
                        PILLAR_LABELS[left],
                        left_cycle.get_name(),
                        PILLAR_LABELS[right],
                        right_cycle.get_name(),
                        left_cycle.get_heaven_stem().get_name(),
                        right_cycle.get_heaven_stem().get_name(),
                        element.get_name()
                    ),
                });
            }
            for relation in branch_relations(
                left_cycle.get_earth_branch(),
                right_cycle.get_earth_branch(),
            ) {
                rows.push(RelationRow {
                    kind: "地支关系".into(),
                    relation: format!(
                        "{}{} 与 {}{}: {relation}",
                        PILLAR_LABELS[left],
                        left_cycle.get_name(),
                        PILLAR_LABELS[right],
                        right_cycle.get_name()
                    ),
                });
            }
        }
    }
    for (branch, labels) in &branch_map {
        if labels.len() > 1 && ["辰", "午", "酉", "亥"].contains(&branch.as_str()) {
            rows.push(RelationRow {
                kind: "地支自刑".into(),
                relation: format!("{}: {branch}{branch}自刑", labels.join("、")),
            });
        }
    }
    let combos: [(&str, [&str; 3], &str); 8] = [
        ("三合", ["申", "子", "辰"], "水"),
        ("三合", ["亥", "卯", "未"], "木"),
        ("三合", ["寅", "午", "戌"], "火"),
        ("三合", ["巳", "酉", "丑"], "金"),
        ("三会", ["寅", "卯", "辰"], "木"),
        ("三会", ["巳", "午", "未"], "火"),
        ("三会", ["申", "酉", "戌"], "金"),
        ("三会", ["亥", "子", "丑"], "水"),
    ];
    for (kind, branches, element) in combos {
        if branches
            .iter()
            .all(|branch| branch_map.iter().any(|(name, _)| name == branch))
        {
            let locations = branches
                .iter()
                .map(|branch| {
                    let labels = branch_map
                        .iter()
                        .find(|(name, _)| name == branch)
                        .map_or_else(String::new, |(_, labels)| labels.join("/"));
                    format!("{branch}:{labels}")
                })
                .collect::<Vec<_>>()
                .join("；");
            rows.push(RelationRow {
                kind: kind.into(),
                relation: format!("{}成{element}局（{locations}）", branches.join("")),
            });
        }
    }
    rows
}

pub(crate) fn build_structure(datetime: &str, gender_input: &str) -> Result<StructureData, String> {
    let ctx = context(datetime)?;
    let chart = build_chart(datetime, gender_input)?;
    let scores = element_scores(&ctx.eight_char);
    let total: f64 = scores.iter().sum();
    let day_element = ctx.day_master.get_element().get_name();
    let element_scores = ELEMENTS
        .iter()
        .enumerate()
        .map(|(index, element)| ElementScore {
            element,
            score: scores[index],
            percent: scores[index] / total * 100.0,
            is_day_master: *element == day_element,
        })
        .collect();
    let mut ten_gods: Vec<(String, f64)> = Vec::new();
    for pillar in &chart.pillars {
        let mut add = |name: &str, value: f64| {
            if let Some((_, score)) = ten_gods.iter_mut().find(|(item, _)| item == name) {
                *score += value;
            } else {
                ten_gods.push((name.into(), value));
            }
        };
        add(&pillar.ten_god, 1.0);
        for hidden in &pillar.hidden_stems {
            add(&hidden.ten_god, hidden.weight);
        }
    }
    ten_gods.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    for (_, score) in &mut ten_gods {
        *score = (*score * 10.0).round() / 10.0;
    }
    let roots = chart
        .pillars
        .iter()
        .enumerate()
        .map(|(index, pillar)| {
            let matches = pillar
                .hidden_stems
                .iter()
                .filter(|hidden| {
                    HeavenStem::from_name(&hidden.stem).get_element().get_name() == day_element
                })
                .cloned()
                .collect::<Vec<_>>();
            RootEvidence {
                pillar: PILLAR_LABELS[index],
                branch: pillar.earth_branch.clone(),
                exact: matches.iter().any(|h| h.stem == ctx.day_master.get_name()),
                matches,
            }
        })
        .collect();
    let visible_stems = [0, 1, 3]
        .into_iter()
        .map(|index| {
            (
                PILLAR_LABELS[index],
                chart.pillars[index].heaven_stem.clone(),
                chart.pillars[index].ten_god.clone(),
            )
        })
        .collect();
    let month = ctx.eight_char.get_month().get_earth_branch();
    Ok(StructureData {
        chart,
        month_branch: month.get_name(),
        month_element: month.get_element().get_name(),
        element_scores,
        ten_gods,
        roots,
        visible_stems,
        relations: relation_rows(&ctx.eight_char),
    })
}

fn pair_summary(left: &SixtyCycle, right: &SixtyCycle) -> String {
    let mut relations = Vec::new();
    if let Some(element) = left.get_heaven_stem().combine(right.get_heaven_stem()) {
        relations.push(format!(
            "{}{}合{}",
            left.get_heaven_stem().get_name(),
            right.get_heaven_stem().get_name(),
            element.get_name()
        ));
    }
    relations.extend(branch_relations(
        left.get_earth_branch(),
        right.get_earth_branch(),
    ));
    if relations.is_empty() {
        "无".into()
    } else {
        relations.join("、")
    }
}

fn original_relations(
    target: &SixtyCycle,
    eight_char: &EightChar,
) -> Vec<(&'static str, String, String)> {
    cycles(eight_char)
        .iter()
        .enumerate()
        .map(|(index, original)| {
            (
                PILLAR_LABELS[index],
                original.get_name(),
                pair_summary(original, target),
            )
        })
        .collect()
}

fn special_relations(target: &SixtyCycle, eight_char: &EightChar) -> String {
    [
        ("胎元", eight_char.get_fetal_origin()),
        ("命宫", eight_char.get_own_sign()),
        ("身宫", reference_body_sign(eight_char)),
    ]
    .iter()
    .map(|(label, cycle)| {
        format!(
            "{label}{}:{}",
            cycle.get_name(),
            pair_summary(cycle, target)
        )
    })
    .collect::<Vec<_>>()
    .join("；")
}

fn evidence(cycle: &SixtyCycle, day_master: &HeavenStem, eight_char: &EightChar) -> CycleEvidence {
    let mut detail = pillar_detail(cycle, day_master, false);
    detail.shensha = shensha::cycle_hits(eight_char, &detail.earth_branch);
    CycleEvidence {
        cycle: detail.pillar,
        ten_god: detail.ten_god,
        heaven_stem: detail.heaven_stem,
        earth_branch: detail.earth_branch,
        hidden_stems: detail.hidden_stems,
        star_fortune: detail.terrain,
        self_sitting: detail.self_sitting,
        kong_wang: detail.kong_wang,
        nayin: detail.nayin,
        shensha: detail.shensha,
    }
}

fn tai_sui(flow: &str, natal: &str) -> Vec<&'static str> {
    let opposite = [
        ("子", "午"),
        ("丑", "未"),
        ("寅", "申"),
        ("卯", "酉"),
        ("辰", "戌"),
        ("巳", "亥"),
        ("午", "子"),
        ("未", "丑"),
        ("申", "寅"),
        ("酉", "卯"),
        ("戌", "辰"),
        ("亥", "巳"),
    ];
    let combine = [
        ("子", "丑"),
        ("丑", "子"),
        ("寅", "亥"),
        ("亥", "寅"),
        ("卯", "戌"),
        ("戌", "卯"),
        ("辰", "酉"),
        ("酉", "辰"),
        ("巳", "申"),
        ("申", "巳"),
        ("午", "未"),
        ("未", "午"),
    ];
    let harm = [
        ("子", "未"),
        ("丑", "午"),
        ("寅", "巳"),
        ("卯", "辰"),
        ("辰", "卯"),
        ("巳", "寅"),
        ("午", "丑"),
        ("未", "子"),
        ("申", "亥"),
        ("酉", "戌"),
        ("戌", "酉"),
        ("亥", "申"),
    ];
    let breaks = [
        ("子", "酉"),
        ("酉", "子"),
        ("丑", "辰"),
        ("辰", "丑"),
        ("寅", "亥"),
        ("亥", "寅"),
        ("卯", "午"),
        ("午", "卯"),
        ("巳", "申"),
        ("申", "巳"),
        ("未", "戌"),
        ("戌", "未"),
    ];
    let mut result = Vec::new();
    if flow == natal {
        result.push("值太岁");
    }
    if opposite.contains(&(flow, natal)) {
        result.push("冲太岁");
    }
    if combine.contains(&(flow, natal)) {
        result.push("合太岁");
    }
    if [["子", "卯", ""], ["寅", "巳", "申"], ["丑", "未", "戌"]]
        .iter()
        .any(|group| group.contains(&flow) && group.contains(&natal))
    {
        result.push("刑太岁");
    }
    if harm.contains(&(flow, natal)) {
        result.push("害太岁");
    }
    if breaks.contains(&(flow, natal)) {
        result.push("破太岁");
    }
    result
}

pub(crate) fn build_timeline(
    datetime: &str,
    gender_input: &str,
    start_year: isize,
    count: usize,
) -> Result<TimelineData, String> {
    let ctx = context(datetime)?;
    if start_year < ctx.parts.year {
        return Err("startYear must be >= birth year.".into());
    }
    let final_year = start_year
        .checked_add(count.saturating_sub(1) as isize)
        .ok_or_else(|| format!("illegal sixty cycle year: {start_year}"))?;
    // Validate both ends before fortune arithmetic; this keeps huge public
    // integers out of the library's infallible `next` implementations.
    sixty_cycle_year_with_range(start_year)?;
    sixty_cycle_year_with_range(final_year)?;
    let chart = build_chart(datetime, gender_input)?;
    let (gender, _) = gender(gender_input)?;
    let child = ChildLimit::from_solar_time(ctx.solar_time, gender);
    let first_fortune = child.get_start_fortune();
    let first_age = first_fortune.get_age();
    let natal_year_branch = ctx.eight_char.get_year().get_earth_branch().get_name();
    let mut rows = Vec::with_capacity(count);
    for offset in 0..count {
        let year = start_year + offset as isize;
        let age = year - ctx.parts.year + 1;
        let fortune = first_fortune.next(age - first_age);
        let sc_year = sixty_cycle_year_with_range(year)?;
        let annual_cycle = sc_year.get_sixty_cycle();
        let mut decade = child.get_start_decade_fortune();
        let mut decade_evidence = None;
        for _ in 0..10 {
            if age >= decade.get_start_age() && age <= decade.get_end_age() {
                decade_evidence = Some(DecadeEvidence {
                    evidence: evidence(&decade.get_sixty_cycle(), &ctx.day_master, &ctx.eight_char),
                    start_age: decade.get_start_age(),
                    end_age: decade.get_end_age(),
                });
                break;
            }
            decade = decade.next(1);
        }
        rows.push(TimelineRow {
            year,
            solar_range: year_range(&sc_year),
            age,
            decade: decade_evidence,
            small_luck: evidence(&fortune.get_sixty_cycle(), &ctx.day_master, &ctx.eight_char),
            annual: evidence(&annual_cycle, &ctx.day_master, &ctx.eight_char),
            tai_sui: tai_sui(
                &annual_cycle.get_earth_branch().get_name(),
                &natal_year_branch,
            ),
            original_relations: original_relations(&annual_cycle, &ctx.eight_char),
            special_relations: special_relations(&annual_cycle, &ctx.eight_char),
        });
    }
    Ok(TimelineData {
        chart,
        start_year,
        count,
        rows,
    })
}

pub(crate) fn build_period(
    datetime: &str,
    gender_input: &str,
    scope: &str,
    year: Option<isize>,
    month: Option<usize>,
    date: Option<&str>,
    hour: Option<usize>,
) -> Result<PeriodData, String> {
    let ctx = context(datetime)?;
    let chart = build_chart(datetime, gender_input)?;
    let mut jie_qi = None;
    let mut twelve_star = None;
    let mut nine_star = None;
    let mut recommends = None;
    let mut avoids = None;
    let mut special = None;
    let mut timeline = None;
    let (label, cycle, range) = match scope {
        "year" => {
            let year = year.ok_or_else(|| "scope=year requires year.".to_string())?;
            if year == 0 {
                return Err("scope=year requires year.".into());
            }
            let sc_year = sixty_cycle_year_with_range(year)?;
            let sc = sc_year.get_sixty_cycle();
            timeline = Some(
                build_timeline(datetime, gender_input, year, 1)?
                    .rows
                    .remove(0),
            );
            (format!("{year}年"), sc, year_range(&sc_year))
        }
        "month" => {
            let year = year.ok_or_else(|| "scope=month requires year and month.".to_string())?;
            let month = month.ok_or_else(|| "scope=month requires year and month.".to_string())?;
            if year == 0 {
                return Err("scope=month requires year and month.".into());
            }
            if !(1..=12).contains(&month) {
                return Err("month must be 1-12.".into());
            }
            let sc_month = sixty_cycle_year_with_range(year)?.get_months()[month - 1].clone();
            let sc = sc_month.get_sixty_cycle();
            jie_qi = Some(
                sc_month
                    .get_first_day()
                    .get_solar_day()
                    .get_term()
                    .get_name(),
            );
            special = Some(special_relations(&sc, &ctx.eight_char));
            (
                format!("{year}年第{month}个干支月"),
                sc,
                month_range(&sc_month),
            )
        }
        "day" => {
            let date = date.ok_or_else(|| "scope=day requires date in YYYY-MM-DD.".to_string())?;
            let (year, month, day) = parse_date(date)
                .map_err(|_| "scope=day requires date in YYYY-MM-DD.".to_string())?;
            validate_period_date_year(year)?;
            tyme4rs::tyme::solar::SolarMonth::new(year, month)?;
            let solar_day = SolarDay::new(year, month, day)?;
            let sc_day = SixtyCycleDay::from_solar_day(solar_day);
            let sc = sc_day.get_sixty_cycle();
            twelve_star = Some(sc_day.get_twelve_star().get_name());
            nine_star = Some(sc_day.get_nine_star().to_string());
            recommends = Some(
                sc_day
                    .get_recommends()
                    .iter()
                    .map(Culture::get_name)
                    .collect(),
            );
            avoids = Some(sc_day.get_avoids().iter().map(Culture::get_name).collect());
            special = Some(special_relations(&sc, &ctx.eight_char));
            (date.into(), sc, day_range(&sc_day))
        }
        "hour" => {
            let date = date.ok_or_else(|| "scope=hour requires date in YYYY-MM-DD.".to_string())?;
            let (year, month, day) = parse_date(date)
                .map_err(|_| "scope=hour requires date in YYYY-MM-DD.".to_string())?;
            let hour = hour
                .filter(|h| *h <= 23)
                .ok_or_else(|| "scope=hour requires hour 0-23.".to_string())?;
            validate_period_date_year(year)?;
            tyme4rs::tyme::solar::SolarMonth::new(year, month)?;
            let time = SolarTime::new(year, month, day, hour, 0, 0)?;
            let sc_hour = time.get_sixty_cycle_hour();
            let sc = sc_hour.get_sixty_cycle();
            twelve_star = Some(sc_hour.get_twelve_star().get_name());
            nine_star = Some(sc_hour.get_nine_star().to_string());
            recommends = Some(
                sc_hour
                    .get_recommends()
                    .iter()
                    .map(Culture::get_name)
                    .collect(),
            );
            avoids = Some(sc_hour.get_avoids().iter().map(Culture::get_name).collect());
            special = Some(special_relations(&sc, &ctx.eight_char));
            (format!("{date} {hour:02}:00"), sc, hour_range(&sc_hour))
        }
        _ => return Err("Unsupported scope. Use year, month, day, or hour.".into()),
    };
    Ok(PeriodData {
        chart,
        scope: scope.into(),
        label,
        evidence: evidence(&cycle, &ctx.day_master, &ctx.eight_char),
        solar_range: range,
        jie_qi,
        twelve_star,
        nine_star,
        recommends,
        avoids,
        relations: original_relations(&cycle, &ctx.eight_char),
        special_relations: special,
        timeline,
    })
}

pub(crate) fn build_shensha(datetime: &str) -> Result<Vec<(String, String, String)>, String> {
    debug_assert_eq!(shensha::SOURCE_TABLE_COUNT, 46);
    let ctx = context(datetime)?;
    let cycles = cycles(&ctx.eight_char);
    let mut rows = Vec::new();
    for (index, names) in shensha::pillar_hits(&ctx.eight_char)
        .into_iter()
        .enumerate()
    {
        for name in names {
            rows.push((
                name,
                shensha::PillarPosition::ALL[index].label().into(),
                cycles[index].get_name(),
            ));
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_datetime_shape_and_calendar_value() {
        assert_eq!(
            parse_datetime("2024/01/15 08:30").unwrap_err(),
            "Invalid format. Use YYYY-MM-DD HH:MM"
        );
        assert!(context("2024-02-30 08:30").is_err());
        assert_eq!(
            context("0000-01-01 00:00").err().as_deref(),
            Some("illegal solar year: 0")
        );
        assert_eq!(
            context("2024-13-01 00:00").err().as_deref(),
            Some("illegal solar month: 13")
        );
    }

    #[test]
    fn hour_ranges_use_double_hour_boundaries() {
        let time = SolarTime::from_ymd_hms(2026, 6, 12, 18, 0, 0).get_sixty_cycle_hour();
        assert_eq!(
            hour_range(&time),
            "2026-06-12 17:00:00 至 2026-06-12 18:59:59"
        );
    }

    #[test]
    fn chart_uses_library_for_known_four_pillars() {
        let chart = build_chart("2024-01-15 08:30", "男").unwrap();
        assert_eq!(chart.four_pillar_text, "癸卯 乙丑 戊寅 丙辰");
        assert_eq!(chart.day_master, "戊");
    }

    #[test]
    fn rejects_years_before_tyme_panicking_convenience_paths() {
        assert_eq!(
            build_timeline("2024-01-15 08:30", "男", 10_000, 1).unwrap_err(),
            "illegal sixty cycle year: 10000"
        );
        assert_eq!(
            build_timeline("2024-01-15 08:30", "男", 9_999, 1).unwrap_err(),
            "illegal sixty cycle year: 10000"
        );
        assert_eq!(
            build_period(
                "2024-01-15 08:30",
                "男",
                "month",
                Some(-2),
                Some(1),
                None,
                None,
            )
            .unwrap_err(),
            "illegal sixty cycle year: -2"
        );
        assert_eq!(
            build_chart("9999-01-15 08:30", "男").unwrap_err(),
            "birth datetime exceeds tyme4rs safe range (year <= 9988)."
        );
        assert_eq!(
            build_chart("9989-10-29 23:30", "男").unwrap_err(),
            "birth datetime exceeds tyme4rs safe range (year <= 9988)."
        );
        assert_eq!(
            build_chart("0001-01-01 00:00", "男").unwrap_err(),
            "illegal solar year: 0"
        );
        assert_eq!(
            build_period(
                "2024-01-15 08:30",
                "男",
                "day",
                None,
                None,
                Some("0000-01-01"),
                None,
            )
            .unwrap_err(),
            "period date exceeds tyme4rs safe range (year 2..=9998)."
        );
    }

    #[test]
    fn calendar_boundaries_are_errors_not_unwinds() {
        for datetime in [
            "0000-01-01 00:00",
            "0001-01-01 00:00",
            "2024-00-01 00:00",
            "2024-13-01 00:00",
            "2024-02-30 00:00",
            "2024-01-01 24:00",
            "2024-01-01 00:60",
            "9990-01-01 00:00",
            "9999-12-31 23:30",
        ] {
            let result = std::panic::catch_unwind(|| build_chart(datetime, "男"));
            assert!(result.is_ok(), "build_chart unwound for {datetime}");
        }

        for date in [
            "0000-01-01",
            "0001-01-01",
            "2024-00-01",
            "2024-13-01",
            "2024-02-30",
            "9999-12-31",
        ] {
            let day = std::panic::catch_unwind(|| {
                build_period(
                    "2024-01-15 08:30",
                    "男",
                    "day",
                    None,
                    None,
                    Some(date),
                    None,
                )
            });
            assert!(day.is_ok(), "day period unwound for {date}");
            let hour = std::panic::catch_unwind(|| {
                build_period(
                    "2024-01-15 08:30",
                    "男",
                    "hour",
                    None,
                    None,
                    Some(date),
                    Some(23),
                )
            });
            assert!(hour.is_ok(), "hour period unwound for {date}");
        }
    }

    #[test]
    fn all_period_edge_year_dates_are_rejected_before_calendar_expansion() {
        for date in ["0001-01-02", "0001-12-31", "9999-01-01", "9999-12-30"] {
            for scope in ["day", "hour"] {
                let result = std::panic::catch_unwind(|| {
                    build_period(
                        "2024-01-15 08:30",
                        "男",
                        scope,
                        None,
                        None,
                        Some(date),
                        (scope == "hour").then_some(23),
                    )
                });
                assert!(result.is_ok(), "{scope} period unwound for {date}");
                assert_eq!(
                    result.expect("catch_unwind checked").unwrap_err(),
                    "period date exceeds tyme4rs safe range (year 2..=9998)."
                );
            }
        }
    }

    #[test]
    fn supported_birth_year_boundaries_never_unwind() {
        for gender in ["男", "女"] {
            for year in [2, 9_988] {
                for month in 1..=12 {
                    for day in 1..=31 {
                        for hour in [0, 23] {
                            let datetime = format!("{year:04}-{month:02}-{day:02} {hour:02}:30");
                            let result =
                                std::panic::catch_unwind(|| build_chart(&datetime, gender));
                            assert!(
                                result.is_ok(),
                                "build_chart unwound for {datetime}/{gender}"
                            );
                        }
                    }
                }
            }
        }
    }
}

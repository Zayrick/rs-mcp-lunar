//! 八字神煞静态规则与命中计算。
//!
//! 数据来源：`taibu-core/data/shensha` v3.4.0（MIT），对应参考项目锁定依赖
//! `taibu-core@3.4.0` 的 46 张被使用数据表。这里仅移植参考项目实际消费的
//! 静态规则，历法与四柱计算继续交给 `tyme4rs`，避免自行维护历法算法。
//! 命中顺序、去重方式以及本命柱/流运分支的差异均与参考实现保持一致。

use tyme4rs::tyme::Culture;
use tyme4rs::tyme::eightchar::EightChar;

/// 本模块从 taibu-core v3.4.0 移植并实际参与计算的数据表数量。
pub const SOURCE_TABLE_COUNT: usize = 46;

/// 四柱位置。顺序固定为年、月、日、时，与 MCP 表格输出顺序一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PillarPosition {
    Year,
    Month,
    Day,
    Hour,
}

impl PillarPosition {
    pub const ALL: [Self; 4] = [Self::Year, Self::Month, Self::Day, Self::Hour];

    pub const fn index(self) -> usize {
        match self {
            Self::Year => 0,
            Self::Month => 1,
            Self::Day => 2,
            Self::Hour => 3,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Year => "年柱",
            Self::Month => "月柱",
            Self::Day => "日柱",
            Self::Hour => "时柱",
        }
    }
}

/// 一个干支柱。使用拥有所有权的字符串，让适配层可安全接收 `tyme4rs` 返回值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pillar {
    pub stem: String,
    pub branch: String,
}

impl Pillar {
    pub fn new(stem: impl Into<String>, branch: impl Into<String>) -> Self {
        Self {
            stem: stem.into(),
            branch: branch.into(),
        }
    }

    pub fn name(&self) -> String {
        format!("{}{}", self.stem, self.branch)
    }
}

/// 神煞计算所需的最小上下文；不包含历法计算逻辑。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaziShenshaContext {
    pub year: Pillar,
    pub month: Pillar,
    pub day: Pillar,
    pub hour: Pillar,
    pub kong_wang_branches: [String; 2],
    pub year_nayin_element: Option<String>,
}

impl BaziShenshaContext {
    fn pillar(&self, position: PillarPosition) -> &Pillar {
        match position {
            PillarPosition::Year => &self.year,
            PillarPosition::Month => &self.month,
            PillarPosition::Day => &self.day,
            PillarPosition::Hour => &self.hour,
        }
    }

    fn stems(&self) -> [&str; 4] {
        [
            &self.year.stem,
            &self.month.stem,
            &self.day.stem,
            &self.hour.stem,
        ]
    }

    fn branches(&self) -> [&str; 4] {
        [
            &self.year.branch,
            &self.month.branch,
            &self.day.branch,
            &self.hour.branch,
        ]
    }
}

/// 将 `tyme4rs` 八字转换成神煞计算所需的最小上下文。
pub fn context_from_eight_char(eight_char: &EightChar) -> BaziShenshaContext {
    let year = eight_char.get_year();
    let month = eight_char.get_month();
    let day = eight_char.get_day();
    let hour = eight_char.get_hour();
    let day_extras = day.get_extra_earth_branches();

    BaziShenshaContext {
        year: Pillar::new(
            year.get_heaven_stem().get_name(),
            year.get_earth_branch().get_name(),
        ),
        month: Pillar::new(
            month.get_heaven_stem().get_name(),
            month.get_earth_branch().get_name(),
        ),
        day: Pillar::new(
            day.get_heaven_stem().get_name(),
            day.get_earth_branch().get_name(),
        ),
        hour: Pillar::new(
            hour.get_heaven_stem().get_name(),
            hour.get_earth_branch().get_name(),
        ),
        kong_wang_branches: [day_extras[0].get_name(), day_extras[1].get_name()],
        year_nayin_element: na_yin_element(&year.get_sound().get_name()).map(str::to_owned),
    }
}

/// 计算本命四柱神煞，返回顺序为年、月、日、时。
pub fn pillar_hits(eight_char: &EightChar) -> [Vec<String>; 4] {
    context_pillar_hits(&context_from_eight_char(eight_char))
        .map(|names| names.into_iter().map(str::to_owned).collect())
}

/// 计算流运干支的分支神煞。
///
/// 与参考实现一致：流运没有柱位提示，因此不加入天罗/地网、三奇和柱位专属规则，
/// 但会附带日柱的魁罡、阴差阳错、十恶大败三项基础判断。
pub fn cycle_hits(eight_char: &EightChar, target_branch: &str) -> Vec<String> {
    context_cycle_hits(&context_from_eight_char(eight_char), target_branch)
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// 对纯业务上下文计算本命四柱神煞，返回值借用静态规则名称，无额外字符串分配。
pub fn context_pillar_hits(context: &BaziShenshaContext) -> [Vec<&'static str>; 4] {
    PillarPosition::ALL.map(|position| {
        let pillar = context.pillar(position);
        let mut names = calculate_branch_shensha(context, &pillar.branch, Some(position));
        add_pillar_specific_shensha(context, position, &mut names);
        names
    })
}

/// 对纯业务上下文计算一个流运分支的神煞。
pub fn context_cycle_hits(context: &BaziShenshaContext, target_branch: &str) -> Vec<&'static str> {
    calculate_branch_shensha(context, target_branch, None)
}

/// 从纳音名称提取五行。taibu-core 学堂表只消费末尾的五行字。
pub fn na_yin_element(name: &str) -> Option<&'static str> {
    match name.chars().last()? {
        '金' => Some("金"),
        '木' => Some("木"),
        '水' => Some("水"),
        '火' => Some("火"),
        '土' => Some("土"),
        _ => None,
    }
}

fn calculate_branch_shensha(
    context: &BaziShenshaContext,
    target_branch: &str,
    position_hint: Option<PillarPosition>,
) -> Vec<&'static str> {
    let mut names = Vec::new();

    match_values(
        TIAN_YI_GUI_REN,
        &context.day.stem,
        target_branch,
        "天乙贵人",
        &mut names,
    );
    match_values(
        TAI_JI_GUI_REN,
        &context.day.stem,
        target_branch,
        "太极贵人",
        &mut names,
    );
    match_values(
        TAI_JI_GUI_REN,
        &context.year.stem,
        target_branch,
        "太极贵人",
        &mut names,
    );
    match_scalar(
        LU_SHEN,
        &context.day.stem,
        target_branch,
        "禄神",
        &mut names,
    );
    match_scalar(
        YANG_REN,
        &context.day.stem,
        target_branch,
        "羊刃",
        &mut names,
    );
    match_scalar(
        WEN_CHANG,
        &context.day.stem,
        target_branch,
        "文昌",
        &mut names,
    );
    match_scalar(
        TIAN_CHU,
        &context.day.stem,
        target_branch,
        "天厨",
        &mut names,
    );
    match_scalar(
        GUO_YIN,
        &context.day.stem,
        target_branch,
        "国印贵人",
        &mut names,
    );
    match_scalar(
        FU_XING,
        &context.day.stem,
        target_branch,
        "福星贵人",
        &mut names,
    );
    match_scalar(
        LIU_XIA,
        &context.day.stem,
        target_branch,
        "流霞",
        &mut names,
    );
    match_scalar(
        HONG_YAN,
        &context.day.stem,
        target_branch,
        "红艳煞",
        &mut names,
    );
    match_scalar(
        FEI_REN,
        &context.day.stem,
        target_branch,
        "飞刃",
        &mut names,
    );
    match_scalar(
        CI_GUAN,
        &context.day.stem,
        target_branch,
        "词馆",
        &mut names,
    );

    for basis_branch in [&context.day.branch, &context.year.branch] {
        match_scalar(YI_MA, basis_branch, target_branch, "驿马", &mut names);
        match_scalar(TAO_HUA, basis_branch, target_branch, "桃花", &mut names);
        match_scalar(HUA_GAI, basis_branch, target_branch, "华盖", &mut names);
        match_scalar(JIANG_XING, basis_branch, target_branch, "将星", &mut names);
        match_scalar(JIE_SHA, basis_branch, target_branch, "劫煞", &mut names);
        match_scalar(WANG_SHEN, basis_branch, target_branch, "亡神", &mut names);
        match_scalar(ZAI_SHA, basis_branch, target_branch, "灾煞", &mut names);
    }

    if let Some(element) = context.year_nayin_element.as_deref() {
        match_scalar(XUE_TANG, element, target_branch, "学堂", &mut names);
    }
    match_scalar(
        HONG_LUAN,
        &context.year.branch,
        target_branch,
        "红鸾",
        &mut names,
    );
    match_scalar(
        TIAN_XI,
        &context.year.branch,
        target_branch,
        "天喜",
        &mut names,
    );
    match_scalar(
        DIAO_KE,
        &context.year.branch,
        target_branch,
        "吊客",
        &mut names,
    );
    match_scalar(
        SANG_MEN,
        &context.year.branch,
        target_branch,
        "丧门",
        &mut names,
    );
    match_scalar(
        PI_TOU,
        &context.year.branch,
        target_branch,
        "披头",
        &mut names,
    );
    match_scalar(
        GOU_SHA,
        &context.year.branch,
        target_branch,
        "勾煞",
        &mut names,
    );
    match_scalar(
        JIAO_SHA,
        &context.year.branch,
        target_branch,
        "绞煞",
        &mut names,
    );
    match_scalar(
        TIAN_YI,
        &context.month.branch,
        target_branch,
        "天医",
        &mut names,
    );
    match_scalar(
        BAI_HU,
        &context.month.branch,
        target_branch,
        "白虎",
        &mut names,
    );
    match_scalar(
        XUE_REN,
        &context.day.branch,
        target_branch,
        "血刃",
        &mut names,
    );

    if lookup_scalar(GU_CHEN, &context.year.branch) == Some(target_branch) {
        add_unique(&mut names, "孤辰");
    }
    if lookup_scalar(GUA_SU, &context.year.branch) == Some(target_branch) {
        add_unique(&mut names, "寡宿");
    }
    if context
        .kong_wang_branches
        .iter()
        .any(|branch| branch == target_branch)
    {
        add_unique(&mut names, "空亡");
    }

    if position_hint.is_some() {
        let all_branches = context.branches();
        if (target_branch == "戌" && all_branches.contains(&"亥"))
            || (target_branch == "亥" && all_branches.contains(&"戌"))
        {
            add_unique(&mut names, "天罗");
        }
        if (target_branch == "辰" && all_branches.contains(&"巳"))
            || (target_branch == "巳" && all_branches.contains(&"辰"))
        {
            add_unique(&mut names, "地网");
        }
    }

    if position_hint == Some(PillarPosition::Day) {
        check_day_pillar_shensha(context, &mut names, true);
    }

    if let Some(position) = position_hint {
        let stems = context.stems();
        for (name, qi_stems) in SAN_QI {
            for index in 0..=stems.len() - 3 {
                if stems[index..index + 3] == **qi_stems
                    && (index..index + 3).contains(&position.index())
                {
                    add_unique(&mut names, name);
                    break;
                }
            }
        }
    } else {
        check_day_pillar_shensha(context, &mut names, false);
    }

    names
}

fn check_day_pillar_shensha(
    context: &BaziShenshaContext,
    names: &mut Vec<&'static str>,
    include_full: bool,
) {
    let day_pillar = context.day.name();
    if KUI_GANG.contains(&day_pillar.as_str()) {
        add_unique(names, "魁罡");
    }
    if YIN_CHA_YANG_CUO.contains(&day_pillar.as_str()) {
        add_unique(names, "阴差阳错");
    }
    if SHI_E_DA_BAI.contains(&day_pillar.as_str()) {
        add_unique(names, "十恶大败");
    }
    if !include_full {
        return;
    }
    if BA_ZHUAN.contains(&day_pillar.as_str()) {
        add_unique(names, "八专");
    }
    if JIN_SHEN.contains(&day_pillar.as_str()) {
        add_unique(names, "金神");
    }
    if GU_LUAN.contains(&day_pillar.as_str()) {
        add_unique(names, "孤鸾煞");
    }
    if lookup_values(SI_FEI_RI, &context.month.branch)
        .is_some_and(|pillars| pillars.contains(&day_pillar.as_str()))
    {
        add_unique(names, "四废");
    }
}

fn add_pillar_specific_shensha(
    context: &BaziShenshaContext,
    position: PillarPosition,
    names: &mut Vec<&'static str>,
) {
    let pillar = context.pillar(position);
    add_stem_or_branch(
        names,
        "金舆",
        lookup_scalar(JIN_YU, &context.day.stem),
        pillar,
    );
    add_stem_or_branch(
        names,
        "月德贵人",
        lookup_scalar(YUE_DE, &context.month.branch),
        pillar,
    );
    add_stem_or_branch(
        names,
        "天德贵人",
        lookup_scalar(TIAN_DE, &context.month.branch),
        pillar,
    );
    if lookup_values(DE_XIU, &context.month.branch)
        .is_some_and(|stems| stems.contains(&pillar.stem.as_str()))
    {
        add_unique(names, "德秀贵人");
    }
    add_stem_or_branch(
        names,
        "天德合",
        lookup_scalar(TIAN_DE_HE, &context.month.branch),
        pillar,
    );
    add_stem_or_branch(
        names,
        "月德合",
        lookup_scalar(YUE_DE_HE, &context.month.branch),
        pillar,
    );
}

fn add_stem_or_branch(
    names: &mut Vec<&'static str>,
    label: &'static str,
    target: Option<&str>,
    pillar: &Pillar,
) {
    if target.is_some_and(|target| target == pillar.stem || target == pillar.branch) {
        add_unique(names, label);
    }
}

fn add_unique(names: &mut Vec<&'static str>, name: &'static str) {
    if !names.contains(&name) {
        names.push(name);
    }
}

type ScalarTable = &'static [(&'static str, &'static str)];
type ValuesTable = &'static [(&'static str, &'static [&'static str])];

fn lookup_scalar(table: ScalarTable, key: &str) -> Option<&'static str> {
    table
        .iter()
        .find_map(|(candidate, value)| (*candidate == key).then_some(*value))
}

fn lookup_values(table: ValuesTable, key: &str) -> Option<&'static [&'static str]> {
    table
        .iter()
        .find_map(|(candidate, values)| (*candidate == key).then_some(*values))
}

fn match_scalar(
    table: ScalarTable,
    key: &str,
    target_branch: &str,
    label: &'static str,
    names: &mut Vec<&'static str>,
) {
    if lookup_scalar(table, key) == Some(target_branch) {
        add_unique(names, label);
    }
}

fn match_values(
    table: ValuesTable,
    key: &str,
    target_branch: &str,
    label: &'static str,
    names: &mut Vec<&'static str>,
) {
    if lookup_values(table, key).is_some_and(|values| values.contains(&target_branch)) {
        add_unique(names, label);
    }
}

// ---- taibu-core/data/shensha v3.4.0: 46 source tables used by the reference ----

const TIAN_YI_GUI_REN: ValuesTable = &[
    ("甲", &["丑", "未"]),
    ("乙", &["子", "申"]),
    ("丙", &["亥", "酉"]),
    ("丁", &["亥", "酉"]),
    ("戊", &["丑", "未"]),
    ("己", &["子", "申"]),
    ("庚", &["丑", "未"]),
    ("辛", &["寅", "午"]),
    ("壬", &["卯", "巳"]),
    ("癸", &["卯", "巳"]),
];

const TAI_JI_GUI_REN: ValuesTable = &[
    ("甲", &["子", "午"]),
    ("乙", &["子", "午"]),
    ("丙", &["卯", "酉"]),
    ("丁", &["卯", "酉"]),
    ("戊", &["辰", "戌", "丑", "未"]),
    ("己", &["辰", "戌", "丑", "未"]),
    ("庚", &["寅", "亥"]),
    ("辛", &["寅", "亥"]),
    ("壬", &["巳", "申"]),
    ("癸", &["巳", "申"]),
];

const YANG_REN: ScalarTable = &[
    ("甲", "卯"),
    ("乙", "辰"),
    ("丙", "午"),
    ("丁", "未"),
    ("戊", "午"),
    ("己", "未"),
    ("庚", "酉"),
    ("辛", "戌"),
    ("壬", "子"),
    ("癸", "丑"),
];

const WEN_CHANG: ScalarTable = &[
    ("甲", "巳"),
    ("乙", "午"),
    ("丙", "申"),
    ("丁", "酉"),
    ("戊", "申"),
    ("己", "酉"),
    ("庚", "亥"),
    ("辛", "子"),
    ("壬", "寅"),
    ("癸", "卯"),
];

const YI_MA: ScalarTable = &[
    ("寅", "申"),
    ("午", "申"),
    ("戌", "申"),
    ("申", "寅"),
    ("子", "寅"),
    ("辰", "寅"),
    ("巳", "亥"),
    ("酉", "亥"),
    ("丑", "亥"),
    ("亥", "巳"),
    ("卯", "巳"),
    ("未", "巳"),
];

const TAO_HUA: ScalarTable = &[
    ("寅", "卯"),
    ("午", "卯"),
    ("戌", "卯"),
    ("申", "酉"),
    ("子", "酉"),
    ("辰", "酉"),
    ("巳", "午"),
    ("酉", "午"),
    ("丑", "午"),
    ("亥", "子"),
    ("卯", "子"),
    ("未", "子"),
];

const HUA_GAI: ScalarTable = &[
    ("寅", "戌"),
    ("午", "戌"),
    ("戌", "戌"),
    ("申", "辰"),
    ("子", "辰"),
    ("辰", "辰"),
    ("巳", "丑"),
    ("酉", "丑"),
    ("丑", "丑"),
    ("亥", "未"),
    ("卯", "未"),
    ("未", "未"),
];

const LU_SHEN: ScalarTable = &[
    ("甲", "寅"),
    ("乙", "卯"),
    ("丙", "巳"),
    ("丁", "午"),
    ("戊", "巳"),
    ("己", "午"),
    ("庚", "申"),
    ("辛", "酉"),
    ("壬", "亥"),
    ("癸", "子"),
];

const JIE_SHA: ScalarTable = &[
    ("寅", "亥"),
    ("午", "亥"),
    ("戌", "亥"),
    ("申", "巳"),
    ("子", "巳"),
    ("辰", "巳"),
    ("巳", "寅"),
    ("酉", "寅"),
    ("丑", "寅"),
    ("亥", "申"),
    ("卯", "申"),
    ("未", "申"),
];

const WANG_SHEN: ScalarTable = &[
    ("寅", "巳"),
    ("午", "巳"),
    ("戌", "巳"),
    ("申", "亥"),
    ("子", "亥"),
    ("辰", "亥"),
    ("巳", "申"),
    ("酉", "申"),
    ("丑", "申"),
    ("亥", "寅"),
    ("卯", "寅"),
    ("未", "寅"),
];

const GU_CHEN: ScalarTable = &[
    ("寅", "巳"),
    ("卯", "巳"),
    ("辰", "巳"),
    ("巳", "申"),
    ("午", "申"),
    ("未", "申"),
    ("申", "亥"),
    ("酉", "亥"),
    ("戌", "亥"),
    ("亥", "寅"),
    ("子", "寅"),
    ("丑", "寅"),
];

const GUA_SU: ScalarTable = &[
    ("寅", "丑"),
    ("卯", "丑"),
    ("辰", "丑"),
    ("巳", "辰"),
    ("午", "辰"),
    ("未", "辰"),
    ("申", "未"),
    ("酉", "未"),
    ("戌", "未"),
    ("亥", "戌"),
    ("子", "戌"),
    ("丑", "戌"),
];

const JIANG_XING: ScalarTable = &[
    ("寅", "午"),
    ("午", "午"),
    ("戌", "午"),
    ("申", "子"),
    ("子", "子"),
    ("辰", "子"),
    ("巳", "酉"),
    ("酉", "酉"),
    ("丑", "酉"),
    ("亥", "卯"),
    ("卯", "卯"),
    ("未", "卯"),
];

const TIAN_CHU: ScalarTable = &[
    ("甲", "巳"),
    ("乙", "午"),
    ("丙", "巳"),
    ("丁", "午"),
    ("戊", "巳"),
    ("己", "午"),
    ("庚", "亥"),
    ("辛", "子"),
    ("壬", "亥"),
    ("癸", "子"),
];

const GUO_YIN: ScalarTable = &[
    ("甲", "戌"),
    ("乙", "亥"),
    ("丙", "丑"),
    ("丁", "寅"),
    ("戊", "丑"),
    ("己", "寅"),
    ("庚", "辰"),
    ("辛", "巳"),
    ("壬", "未"),
    ("癸", "申"),
];

const XUE_TANG: ScalarTable = &[
    ("金", "巳"),
    ("木", "亥"),
    ("水", "申"),
    ("土", "申"),
    ("火", "寅"),
];

const CI_GUAN: ScalarTable = &[
    ("甲", "寅"),
    ("乙", "卯"),
    ("丙", "巳"),
    ("丁", "午"),
    ("戊", "辰"),
    ("己", "未"),
    ("庚", "申"),
    ("辛", "酉"),
    ("壬", "亥"),
    ("癸", "子"),
];

const HONG_LUAN: ScalarTable = &[
    ("子", "卯"),
    ("丑", "寅"),
    ("寅", "丑"),
    ("卯", "子"),
    ("辰", "亥"),
    ("巳", "戌"),
    ("午", "酉"),
    ("未", "申"),
    ("申", "未"),
    ("酉", "午"),
    ("戌", "巳"),
    ("亥", "辰"),
];

const TIAN_XI: ScalarTable = &[
    ("子", "酉"),
    ("丑", "申"),
    ("寅", "未"),
    ("卯", "午"),
    ("辰", "巳"),
    ("巳", "辰"),
    ("午", "卯"),
    ("未", "寅"),
    ("申", "丑"),
    ("酉", "子"),
    ("戌", "亥"),
    ("亥", "戌"),
];

const TIAN_YI: ScalarTable = &[
    ("寅", "丑"),
    ("卯", "寅"),
    ("辰", "卯"),
    ("巳", "辰"),
    ("午", "巳"),
    ("未", "午"),
    ("申", "未"),
    ("酉", "申"),
    ("戌", "酉"),
    ("亥", "戌"),
    ("子", "亥"),
    ("丑", "子"),
];

const DIAO_KE: ScalarTable = &[
    ("子", "戌"),
    ("丑", "亥"),
    ("寅", "子"),
    ("卯", "丑"),
    ("辰", "寅"),
    ("巳", "卯"),
    ("午", "辰"),
    ("未", "巳"),
    ("申", "午"),
    ("酉", "未"),
    ("戌", "申"),
    ("亥", "酉"),
];

const SANG_MEN: ScalarTable = &[
    ("子", "寅"),
    ("丑", "卯"),
    ("寅", "辰"),
    ("卯", "巳"),
    ("辰", "午"),
    ("巳", "未"),
    ("午", "申"),
    ("未", "酉"),
    ("申", "戌"),
    ("酉", "亥"),
    ("戌", "子"),
    ("亥", "丑"),
];

const XUE_REN: ScalarTable = &[
    ("子", "酉"),
    ("丑", "戌"),
    ("寅", "亥"),
    ("卯", "子"),
    ("辰", "丑"),
    ("巳", "寅"),
    ("午", "卯"),
    ("未", "辰"),
    ("申", "巳"),
    ("酉", "午"),
    ("戌", "未"),
    ("亥", "申"),
];

const PI_TOU: ScalarTable = &[
    ("子", "巳"),
    ("丑", "午"),
    ("寅", "未"),
    ("卯", "申"),
    ("辰", "酉"),
    ("巳", "戌"),
    ("午", "亥"),
    ("未", "子"),
    ("申", "丑"),
    ("酉", "寅"),
    ("戌", "卯"),
    ("亥", "辰"),
];

const FU_XING: ScalarTable = &[
    ("甲", "寅"),
    ("乙", "丑"),
    ("丙", "子"),
    ("丁", "亥"),
    ("戊", "申"),
    ("己", "未"),
    ("庚", "午"),
    ("辛", "巳"),
    ("壬", "辰"),
    ("癸", "卯"),
];

const ZAI_SHA: ScalarTable = &[
    ("寅", "子"),
    ("午", "子"),
    ("戌", "子"),
    ("申", "午"),
    ("子", "午"),
    ("辰", "午"),
    ("巳", "卯"),
    ("酉", "卯"),
    ("丑", "卯"),
    ("亥", "酉"),
    ("卯", "酉"),
    ("未", "酉"),
];

const LIU_XIA: ScalarTable = &[
    ("甲", "酉"),
    ("乙", "戌"),
    ("丙", "未"),
    ("丁", "申"),
    ("戊", "巳"),
    ("己", "午"),
    ("庚", "辰"),
    ("辛", "卯"),
    ("壬", "亥"),
    ("癸", "寅"),
];

const HONG_YAN: ScalarTable = &[
    ("甲", "午"),
    ("乙", "午"),
    ("丙", "寅"),
    ("丁", "未"),
    ("戊", "辰"),
    ("己", "辰"),
    ("庚", "戌"),
    ("辛", "酉"),
    ("壬", "子"),
    ("癸", "申"),
];

const GOU_SHA: ScalarTable = &[
    ("子", "酉"),
    ("丑", "戌"),
    ("寅", "亥"),
    ("卯", "子"),
    ("辰", "丑"),
    ("巳", "寅"),
    ("午", "卯"),
    ("未", "辰"),
    ("申", "巳"),
    ("酉", "午"),
    ("戌", "未"),
    ("亥", "申"),
];

const JIAO_SHA: ScalarTable = &[
    ("子", "卯"),
    ("丑", "寅"),
    ("寅", "丑"),
    ("卯", "子"),
    ("辰", "亥"),
    ("巳", "戌"),
    ("午", "酉"),
    ("未", "申"),
    ("申", "未"),
    ("酉", "午"),
    ("戌", "巳"),
    ("亥", "辰"),
];

const BAI_HU: ScalarTable = &[
    ("寅", "午"),
    ("卯", "未"),
    ("辰", "申"),
    ("巳", "酉"),
    ("午", "戌"),
    ("未", "亥"),
    ("申", "子"),
    ("酉", "丑"),
    ("戌", "寅"),
    ("亥", "卯"),
    ("子", "辰"),
    ("丑", "巳"),
];

const FEI_REN: ScalarTable = &[
    ("甲", "酉"),
    ("乙", "戌"),
    ("丙", "子"),
    ("丁", "丑"),
    ("戊", "子"),
    ("己", "丑"),
    ("庚", "卯"),
    ("辛", "辰"),
    ("壬", "午"),
    ("癸", "未"),
];

const KUI_GANG: &[&str] = &["庚辰", "庚戌", "壬辰", "戊戌"];
const YIN_CHA_YANG_CUO: &[&str] = &[
    "丙子", "丁丑", "戊寅", "辛卯", "壬辰", "癸巳", "丙午", "丁未", "戊申", "辛酉", "壬戌", "癸亥",
];
const SHI_E_DA_BAI: &[&str] = &[
    "甲辰", "乙巳", "壬申", "丙申", "丁亥", "庚辰", "戊戌", "癸亥", "辛巳", "己丑",
];
const BA_ZHUAN: &[&str] = &[
    "甲寅", "乙卯", "丙午", "丁未", "戊戌", "戊辰", "己未", "己丑", "庚申", "辛酉", "壬子", "癸丑",
];
const JIN_SHEN: &[&str] = &["乙丑", "己巳", "癸酉"];
const GU_LUAN: &[&str] = &[
    "乙巳", "丁巳", "辛亥", "戊申", "甲寅", "丙午", "戊午", "壬子",
];

const YUE_DE: ScalarTable = &[
    ("寅", "丙"),
    ("午", "丙"),
    ("戌", "丙"),
    ("申", "壬"),
    ("子", "壬"),
    ("辰", "壬"),
    ("亥", "甲"),
    ("卯", "甲"),
    ("未", "甲"),
    ("巳", "庚"),
    ("酉", "庚"),
    ("丑", "庚"),
];

const TIAN_DE: ScalarTable = &[
    ("寅", "丁"),
    ("卯", "申"),
    ("辰", "壬"),
    ("巳", "辛"),
    ("午", "亥"),
    ("未", "甲"),
    ("申", "癸"),
    ("酉", "寅"),
    ("戌", "丙"),
    ("亥", "乙"),
    ("子", "巳"),
    ("丑", "庚"),
];

const JIN_YU: ScalarTable = &[
    ("甲", "辰"),
    ("乙", "巳"),
    ("丙", "未"),
    ("丁", "申"),
    ("戊", "未"),
    ("己", "申"),
    ("庚", "戌"),
    ("辛", "亥"),
    ("壬", "丑"),
    ("癸", "寅"),
];

const DE_XIU: ValuesTable = &[
    ("寅", &["丙", "甲"]),
    ("卯", &["甲", "乙"]),
    ("辰", &["壬", "癸"]),
    ("巳", &["丙", "庚"]),
    ("午", &["丁", "己"]),
    ("未", &["甲", "己"]),
    ("申", &["庚", "壬"]),
    ("酉", &["辛", "庚"]),
    ("戌", &["丙", "戊"]),
    ("亥", &["壬", "甲"]),
    ("子", &["癸", "壬"]),
    ("丑", &["辛", "己"]),
];

const TIAN_DE_HE: ScalarTable = &[
    ("寅", "壬"),
    ("卯", "癸"),
    ("辰", "丁"),
    ("巳", "丙"),
    ("午", "寅"),
    ("未", "己"),
    ("申", "戊"),
    ("酉", "丁"),
    ("戌", "辛"),
    ("亥", "庚"),
    ("子", "庚"),
    ("丑", "乙"),
];

const YUE_DE_HE: ScalarTable = &[
    ("寅", "辛"),
    ("午", "辛"),
    ("戌", "辛"),
    ("申", "丁"),
    ("子", "丁"),
    ("辰", "丁"),
    ("亥", "己"),
    ("卯", "己"),
    ("未", "己"),
    ("巳", "乙"),
    ("酉", "乙"),
    ("丑", "乙"),
];

const SI_FEI_RI: ValuesTable = &[
    ("寅", &["庚申", "辛酉"]),
    ("卯", &["庚申", "辛酉"]),
    ("辰", &["庚申", "辛酉"]),
    ("巳", &["壬子", "癸亥"]),
    ("午", &["壬子", "癸亥"]),
    ("未", &["壬子", "癸亥"]),
    ("申", &["甲寅", "乙卯"]),
    ("酉", &["甲寅", "乙卯"]),
    ("戌", &["甲寅", "乙卯"]),
    ("亥", &["丙午", "丁巳"]),
    ("子", &["丙午", "丁巳"]),
    ("丑", &["丙午", "丁巳"]),
];

const SAN_QI: ValuesTable = &[
    ("天三奇", &["甲", "戊", "庚"]),
    ("地三奇", &["乙", "丙", "丁"]),
    ("人三奇", &["壬", "癸", "辛"]),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn context(
        year: (&str, &str),
        month: (&str, &str),
        day: (&str, &str),
        hour: (&str, &str),
        kong_wang: [&str; 2],
        year_nayin_element: Option<&str>,
    ) -> BaziShenshaContext {
        BaziShenshaContext {
            year: Pillar::new(year.0, year.1),
            month: Pillar::new(month.0, month.1),
            day: Pillar::new(day.0, day.1),
            hour: Pillar::new(hour.0, hour.1),
            kong_wang_branches: kong_wang.map(str::to_owned),
            year_nayin_element: year_nayin_element.map(str::to_owned),
        }
    }

    #[test]
    fn source_version_contract_covers_all_46_reference_tables() {
        assert_eq!(SOURCE_TABLE_COUNT, 46);
        assert_eq!(
            lookup_values(TIAN_YI_GUI_REN, "甲"),
            Some(&["丑", "未"][..])
        );
        assert_eq!(lookup_scalar(FEI_REN, "癸"), Some("未"));
        assert_eq!(lookup_values(SI_FEI_RI, "子"), Some(&["丙午", "丁巳"][..]));
        assert_eq!(
            lookup_values(SAN_QI, "人三奇"),
            Some(&["壬", "癸", "辛"][..])
        );
    }

    #[test]
    fn pillar_hit_order_and_deduplication_match_reference() {
        let context = context(
            ("甲", "子"),
            ("戊", "寅"),
            ("庚", "辰"),
            ("庚", "戌"),
            ["寅", "卯"],
            Some("金"),
        );

        let [year, month, day, hour] = context_pillar_hits(&context);
        assert_eq!(year, vec!["太极贵人", "将星", "天三奇", "德秀贵人"]);
        assert_eq!(
            month,
            vec!["太极贵人", "驿马", "丧门", "孤辰", "空亡", "天三奇"]
        );
        assert_eq!(
            day,
            vec!["国印贵人", "流霞", "华盖", "魁罡", "十恶大败", "天三奇",]
        );
        assert_eq!(hour, vec!["红艳煞", "吊客", "寡宿", "金舆"]);
    }

    #[test]
    fn cycle_hits_keep_only_non_positional_rules_and_basic_day_rules() {
        let context = context(
            ("甲", "子"),
            ("戊", "寅"),
            ("庚", "辰"),
            ("辛", "亥"),
            ["寅", "卯"],
            Some("金"),
        );

        let hits = context_cycle_hits(&context, "辰");
        assert_eq!(hits, vec!["国印贵人", "流霞", "华盖", "魁罡", "十恶大败"]);
        assert!(!hits.contains(&"地网"));
        assert!(!hits.contains(&"金舆"));
    }

    #[test]
    fn day_only_rules_and_four_waste_do_not_leak_to_other_pillars() {
        let context = context(
            ("壬", "子"),
            ("癸", "巳"),
            ("壬", "子"),
            ("甲", "子"),
            ["寅", "卯"],
            Some("木"),
        );

        let [year, month, day, hour] = context_pillar_hits(&context);
        assert!(day.contains(&"四废"));
        assert!(day.contains(&"八专"));
        assert!(day.contains(&"孤鸾煞"));
        for non_day in [&year, &month, &hour] {
            assert!(!non_day.contains(&"四废"));
            assert!(!non_day.contains(&"八专"));
            assert!(!non_day.contains(&"孤鸾煞"));
        }
    }

    #[test]
    fn three_wonders_apply_to_each_pillar_in_the_matching_window() {
        let context = context(
            ("甲", "子"),
            ("戊", "丑"),
            ("庚", "午"),
            ("癸", "酉"),
            ["辰", "巳"],
            Some("金"),
        );
        let hits = context_pillar_hits(&context);

        assert!(hits[0].contains(&"天三奇"));
        assert!(hits[1].contains(&"天三奇"));
        assert!(hits[2].contains(&"天三奇"));
        assert!(!hits[3].contains(&"天三奇"));
    }

    #[test]
    fn tian_luo_di_wang_require_a_natal_position_hint() {
        let context = context(
            ("甲", "戌"),
            ("乙", "亥"),
            ("丙", "辰"),
            ("丁", "巳"),
            ["申", "酉"],
            Some("火"),
        );
        let hits = context_pillar_hits(&context);

        assert!(hits[0].contains(&"天罗"));
        assert!(hits[1].contains(&"天罗"));
        assert!(hits[2].contains(&"地网"));
        assert!(hits[3].contains(&"地网"));
        assert!(!context_cycle_hits(&context, "戌").contains(&"天罗"));
        assert!(!context_cycle_hits(&context, "辰").contains(&"地网"));
    }

    #[test]
    fn extracts_only_valid_nayin_elements() {
        assert_eq!(na_yin_element("海中金"), Some("金"));
        assert_eq!(na_yin_element("大林木"), Some("木"));
        assert_eq!(na_yin_element(""), None);
        assert_eq!(na_yin_element("未知"), None);
    }

    #[test]
    fn direct_tyme_api_matches_context_api() {
        let eight_char = EightChar::new("甲子", "戊寅", "庚辰", "庚戌");
        let expected: [Vec<String>; 4] = context_pillar_hits(&context_from_eight_char(&eight_char))
            .map(|names| names.into_iter().map(str::to_owned).collect());
        assert_eq!(pillar_hits(&eight_char), expected);

        let expected_cycle: Vec<String> =
            context_cycle_hits(&context_from_eight_char(&eight_char), "辰")
                .into_iter()
                .map(str::to_owned)
                .collect();
        assert_eq!(cycle_hits(&eight_char, "辰"), expected_cycle);
    }

    #[test]
    fn reference_mcp_golden_case_matches_exactly() {
        // Reference bazi_shensha("2024-01-15 08:30") uses these four pillars.
        let eight_char = EightChar::new("癸卯", "乙丑", "戊寅", "丙辰");
        let expected: [Vec<String>; 4] = [
            vec!["桃花".into(), "将星".into()],
            vec![
                "天乙贵人".into(),
                "太极贵人".into(),
                "国印贵人".into(),
                "吊客".into(),
                "寡宿".into(),
                "天德合".into(),
                "月德合".into(),
            ],
            vec!["亡神".into(), "阴差阳错".into()],
            vec!["太极贵人".into(), "红艳煞".into(), "词馆".into()],
        ];
        assert_eq!(pillar_hits(&eight_char), expected);
    }
}

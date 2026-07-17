#[derive(Debug, Clone)]
pub(crate) struct HiddenStem {
    pub stem: String,
    pub qi_type: &'static str,
    pub weight: f64,
    pub ten_god: String,
}

#[derive(Debug, Clone)]
pub(crate) struct PillarInfo {
    pub pillar: String,
    pub ten_god: String,
    pub heaven_stem: String,
    pub earth_branch: String,
    pub hidden_stems: Vec<HiddenStem>,
    pub terrain: String,
    pub self_sitting: String,
    pub kong_wang: String,
    pub nayin: String,
    pub shensha: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct TermInfo {
    pub name: String,
    pub time: String,
}

#[derive(Debug, Clone)]
pub(crate) struct LuckRow {
    pub label: String,
    pub cycle: Option<String>,
    pub ten_god: Option<String>,
    pub nayin: Option<String>,
    pub start_age: isize,
    pub end_age: isize,
    pub start_year: isize,
    pub end_year: isize,
}

#[derive(Debug, Clone)]
pub(crate) struct ChartData {
    pub input: String,
    pub gender: String,
    pub solar: String,
    pub lunar: String,
    pub four_pillar_text: String,
    pub day_master: String,
    pub day_kong_wang: String,
    pub commander: String,
    pub pillars: [PillarInfo; 4],
    pub current_jie: TermInfo,
    pub next_jie: TermInfo,
    pub fetal_origin: String,
    pub fetal_breath: String,
    pub life_palace: String,
    pub body_palace: String,
    pub start_luck_description: String,
    pub start_luck_date: String,
    pub is_forward: bool,
    pub luck: Vec<LuckRow>,
}

#[derive(Debug, Clone)]
pub(crate) struct ElementScore {
    pub element: &'static str,
    pub score: f64,
    pub percent: f64,
    pub is_day_master: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RootEvidence {
    pub pillar: &'static str,
    pub branch: String,
    pub matches: Vec<HiddenStem>,
    pub exact: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RelationRow {
    pub kind: String,
    pub relation: String,
}

#[derive(Debug, Clone)]
pub(crate) struct StructureData {
    pub chart: ChartData,
    pub month_branch: String,
    pub month_element: String,
    pub element_scores: Vec<ElementScore>,
    pub ten_gods: Vec<(String, f64)>,
    pub roots: Vec<RootEvidence>,
    pub visible_stems: Vec<(&'static str, String, String)>,
    pub relations: Vec<RelationRow>,
}

#[derive(Debug, Clone)]
pub(crate) struct CycleEvidence {
    pub cycle: String,
    pub ten_god: String,
    pub heaven_stem: String,
    pub earth_branch: String,
    pub hidden_stems: Vec<HiddenStem>,
    pub star_fortune: String,
    pub self_sitting: String,
    pub kong_wang: String,
    pub nayin: String,
    pub shensha: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct DecadeEvidence {
    pub evidence: CycleEvidence,
    pub start_age: isize,
    pub end_age: isize,
}

#[derive(Debug, Clone)]
pub(crate) struct TimelineRow {
    pub year: isize,
    pub solar_range: String,
    pub age: isize,
    pub decade: Option<DecadeEvidence>,
    pub small_luck: CycleEvidence,
    pub annual: CycleEvidence,
    pub tai_sui: Vec<&'static str>,
    pub original_relations: Vec<(&'static str, String, String)>,
    pub special_relations: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TimelineData {
    pub chart: ChartData,
    pub start_year: isize,
    pub count: usize,
    pub rows: Vec<TimelineRow>,
}

#[derive(Debug, Clone)]
pub(crate) struct PeriodData {
    pub chart: ChartData,
    pub scope: String,
    pub label: String,
    pub evidence: CycleEvidence,
    pub solar_range: String,
    pub jie_qi: Option<String>,
    pub twelve_star: Option<String>,
    pub nine_star: Option<String>,
    pub recommends: Option<Vec<String>>,
    pub avoids: Option<Vec<String>>,
    pub relations: Vec<(&'static str, String, String)>,
    pub special_relations: Option<String>,
    pub timeline: Option<TimelineRow>,
}

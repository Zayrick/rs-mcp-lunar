use iztro::core::HoroscopeRuntime;
use iztro::core::labels::chinese_date;
use iztro::{
    BirthTime, Chart, ChartAlgorithmKind, ChartError, Gender, HeavenlyStem, HoroscopeChart,
    HoroscopeLunarDate, HoroscopeSolarDate, HoroscopeStackInput, HoroscopeTargetContext,
    LunarChartRequest, LunarDay as IztroLunarDay, LunarMonth as IztroLunarMonth, MethodProfile,
    MutagenActivation, NominalAgeBoundary, PalaceName, Scope, SolarChartRequest, SolarDay,
    SolarMonth, StaticChartViewSnapshot, StemBranch, TemporalContext, TemporalLayer,
    TemporalPalaceLayout, TemporalPalaceName, WesternZodiac, YearBoundary, birth_year_star_mutagen,
    build_age_horoscope_layer, build_age_period, build_daily_horoscope_layer, build_daily_period,
    build_flow_star_layer, build_full_horoscope_chart, build_hourly_horoscope_layer,
    build_hourly_period, build_monthly_horoscope_layer, build_monthly_period,
    build_yearly_horoscope_layer, build_yearly_period, by_lunar, by_solar,
};
use serde_json::{Map, Value};
use tyme4rs::tyme::Tyme;
use tyme4rs::tyme::lunar::LunarHour;
use tyme4rs::tyme::sixtycycle::SixtyCycleMonth;
use tyme4rs::tyme::solar::{SolarTerm, SolarTime};

use super::locale::Language;

const PROFILES: [&str; 2] = ["sanhe", "feixing-sihua"];
const CALENDARS: [&str; 2] = ["solar", "lunar"];
const SCOPES: [&str; 6] = ["decadal", "age", "yearly", "monthly", "daily", "hourly"];
const TOPICS: [&str; 6] = [
    "self",
    "career",
    "wealth",
    "relationship",
    "health",
    "family",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ZiweiProfile {
    Sanhe,
    FeixingSihua,
}

impl ZiweiProfile {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Sanhe => "sanhe",
            Self::FeixingSihua => "feixing-sihua",
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Sanhe => "三合",
            Self::FeixingSihua => "飞星四化",
        }
    }

    const fn algorithm(self) -> ChartAlgorithmKind {
        match self {
            Self::Sanhe => ChartAlgorithmKind::QuanShu,
            Self::FeixingSihua => ChartAlgorithmKind::Zhongzhou,
        }
    }

    const fn year_boundary(self) -> YearBoundary {
        match self {
            Self::Sanhe => YearBoundary::ChineseNewYearEve,
            Self::FeixingSihua => YearBoundary::LiChun,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ZiweiCalendar {
    Solar,
    Lunar,
}

impl ZiweiCalendar {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Solar => "公历",
            Self::Lunar => "农历",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ZiweiScope {
    Decadal,
    Age,
    Yearly,
    Monthly,
    Daily,
    Hourly,
}

impl ZiweiScope {
    pub(crate) const ALL: [Self; 6] = [
        Self::Decadal,
        Self::Age,
        Self::Yearly,
        Self::Monthly,
        Self::Daily,
        Self::Hourly,
    ];

    pub(crate) const fn iztro(self) -> Scope {
        match self {
            Self::Decadal => Scope::Decadal,
            Self::Age => Scope::Age,
            Self::Yearly => Scope::Yearly,
            Self::Monthly => Scope::Monthly,
            Self::Daily => Scope::Daily,
            Self::Hourly => Scope::Hourly,
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Decadal => "大限",
            Self::Age => "小限",
            Self::Yearly => "流年",
            Self::Monthly => "流月",
            Self::Daily => "流日",
            Self::Hourly => "流时",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ZiweiTopic {
    SelfAnalysis,
    Career,
    Wealth,
    Relationship,
    Health,
    Family,
}

impl ZiweiTopic {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::SelfAnalysis => "self",
            Self::Career => "career",
            Self::Wealth => "wealth",
            Self::Relationship => "relationship",
            Self::Health => "health",
            Self::Family => "family",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DateTimeParts {
    pub(crate) year: i32,
    pub(crate) month: u8,
    pub(crate) day: u8,
    pub(crate) hour: u8,
    pub(crate) minute: u8,
}

impl DateTimeParts {
    pub(crate) const fn time_index(self) -> u8 {
        match self.hour {
            0 => 0,
            23 => 12,
            hour => hour.div_ceil(2),
        }
    }

    fn solar_time(self) -> Result<SolarTime, String> {
        // Converting a civil date in solar year 1 to its lunar facts can step
        // into unsupported year zero (for example 0001-01-01). Keep the whole
        // edge year out instead of relying on date-specific panicking paths.
        if !(2..=9_999).contains(&self.year) {
            return Err("solar datetime exceeds tyme4rs safe range (year 2..=9999).".to_owned());
        }
        // SolarDay::validate in tyme4rs 1.5.0 calls the panicking
        // SolarMonth::from_ym helper before it reports an invalid year/month.
        // Validate the parent unit through its fallible constructor first.
        tyme4rs::tyme::solar::SolarMonth::new(self.year as isize, self.month as usize)?;
        SolarTime::new(
            self.year as isize,
            self.month as usize,
            self.day as usize,
            self.hour as usize,
            self.minute as usize,
            0,
        )
    }
}

pub(crate) struct ChartContext {
    pub(crate) input_datetime: String,
    pub(crate) profile: ZiweiProfile,
    pub(crate) calendar: ZiweiCalendar,
    pub(crate) language: Language,
    pub(crate) time_index: u8,
    birth_leap_layout_offset: isize,
    pub(crate) solar_label: String,
    pub(crate) lunar_label: String,
    pub(crate) chinese_date: [StemBranch; 4],
    pub(crate) western_zodiac: Option<WesternZodiac>,
    pub(crate) chart: Chart,
    pub(crate) view: StaticChartViewSnapshot,
}

pub(crate) struct HoroscopeContext {
    pub(crate) natal: ChartContext,
    pub(crate) birth_datetime: String,
    pub(crate) target_datetime: String,
    pub(crate) target_time_index: u8,
    pub(crate) target_solar_time: SolarTime,
    pub(crate) target_solar_label: String,
    pub(crate) target_lunar_label: String,
    pub(crate) horoscope: HoroscopeChart,
}

impl HoroscopeContext {
    pub(crate) fn runtime(&self) -> Result<HoroscopeRuntime<'_>, String> {
        HoroscopeRuntime::new(&self.horoscope).map_err(|error| error.to_string())
    }
}

#[derive(Debug)]
pub(crate) struct CommonChartArgs {
    pub(crate) datetime: String,
    pub(crate) gender: Gender,
    pub(crate) profile: ZiweiProfile,
    pub(crate) calendar: ZiweiCalendar,
    pub(crate) is_leap_month: bool,
    pub(crate) language: Language,
}

#[derive(Debug)]
pub(crate) struct HoroscopeArgs {
    pub(crate) birth: CommonChartArgs,
    pub(crate) target_datetime: String,
}

pub(crate) fn common_chart_args(
    arguments: &Map<String, Value>,
    datetime_key: &str,
) -> Result<CommonChartArgs, String> {
    Ok(CommonChartArgs {
        datetime: required_string(arguments, datetime_key)?.to_owned(),
        gender: normalize_gender(required_string(arguments, "gender")?)?,
        profile: normalize_profile(optional_string(arguments, "profile")?)?,
        calendar: normalize_calendar(optional_string(arguments, "calendar")?)?,
        is_leap_month: optional_bool(arguments, "isLeapMonth")?.unwrap_or(false),
        language: normalize_language(optional_string(arguments, "language")?)?,
    })
}

pub(crate) fn horoscope_args(arguments: &Map<String, Value>) -> Result<HoroscopeArgs, String> {
    Ok(HoroscopeArgs {
        birth: common_chart_args(arguments, "birthDatetime")?,
        target_datetime: required_string(arguments, "targetDatetime")?.to_owned(),
    })
}

pub(crate) fn required_string<'a>(
    arguments: &'a Map<String, Value>,
    key: &str,
) -> Result<&'a str, String> {
    match arguments.get(key) {
        Some(Value::String(value)) => Ok(value),
        Some(_) => Err(format!("Invalid {key}. Expected a string.")),
        None => Err(format!("Missing required argument: {key}")),
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

fn optional_bool(arguments: &Map<String, Value>, key: &str) -> Result<Option<bool>, String> {
    match arguments.get(key) {
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(format!("Invalid {key}. Expected a boolean.")),
    }
}

pub(crate) fn normalize_scope(value: &str) -> Result<ZiweiScope, String> {
    match value {
        "decadal" => Ok(ZiweiScope::Decadal),
        "age" => Ok(ZiweiScope::Age),
        "yearly" => Ok(ZiweiScope::Yearly),
        "monthly" => Ok(ZiweiScope::Monthly),
        "daily" => Ok(ZiweiScope::Daily),
        "hourly" => Ok(ZiweiScope::Hourly),
        _ => Err(format!(
            "Unsupported scope. Use {}.",
            SCOPES.join(", ").replace(", hourly", ", or hourly")
        )),
    }
}

pub(crate) fn normalize_topic(value: &str) -> Result<ZiweiTopic, String> {
    match value {
        "self" => Ok(ZiweiTopic::SelfAnalysis),
        "career" => Ok(ZiweiTopic::Career),
        "wealth" => Ok(ZiweiTopic::Wealth),
        "relationship" => Ok(ZiweiTopic::Relationship),
        "health" => Ok(ZiweiTopic::Health),
        "family" => Ok(ZiweiTopic::Family),
        _ => Err(format!(
            "Unsupported topic. Use {}.",
            TOPICS.join(", ").replace(", family", ", or family")
        )),
    }
}

fn normalize_profile(value: Option<&str>) -> Result<ZiweiProfile, String> {
    match value.unwrap_or("sanhe") {
        "sanhe" => Ok(ZiweiProfile::Sanhe),
        "feixing-sihua" => Ok(ZiweiProfile::FeixingSihua),
        _ => Err(format!(
            "Unsupported profile. Use {} or {}.",
            PROFILES[0], PROFILES[1]
        )),
    }
}

fn normalize_calendar(value: Option<&str>) -> Result<ZiweiCalendar, String> {
    match value.unwrap_or("solar").to_lowercase().as_str() {
        "solar" | "公历" | "阳历" => Ok(ZiweiCalendar::Solar),
        "lunar" | "农历" | "阴历" => Ok(ZiweiCalendar::Lunar),
        _ => Err(format!(
            "Unsupported calendar. Use {} or {}.",
            CALENDARS[0], CALENDARS[1]
        )),
    }
}

fn normalize_language(value: Option<&str>) -> Result<Language, String> {
    Language::parse(value.unwrap_or("zh-CN"))
}

fn normalize_gender(value: &str) -> Result<Gender, String> {
    match value.trim().to_lowercase().as_str() {
        "男" | "male" | "m" => Ok(Gender::Male),
        "女" | "female" | "f" => Ok(Gender::Female),
        _ => Err("Invalid gender. Use 男/女 or male/female.".to_owned()),
    }
}

pub(crate) fn parse_datetime(value: &str) -> Result<DateTimeParts, String> {
    if value != value.trim() {
        return Err("Invalid format. Use YYYY-MM-DD HH:MM".to_owned());
    }
    let mut fields = value.split_whitespace();
    let date = fields.next();
    let time = fields.next();
    if fields.next().is_some() {
        return Err("Invalid format. Use YYYY-MM-DD HH:MM".to_owned());
    }
    let (Some(date), Some(time)) = (date, time) else {
        return Err("Invalid format. Use YYYY-MM-DD HH:MM".to_owned());
    };
    if date.len() != 10
        || time.len() != 5
        || date.as_bytes()[4] != b'-'
        || date.as_bytes()[7] != b'-'
        || time.as_bytes()[2] != b':'
    {
        return Err("Invalid format. Use YYYY-MM-DD HH:MM".to_owned());
    }
    let numeric = [
        &date[0..4],
        &date[5..7],
        &date[8..10],
        &time[0..2],
        &time[3..5],
    ];
    if numeric
        .iter()
        .any(|part| !part.as_bytes().iter().all(u8::is_ascii_digit))
    {
        return Err("Invalid format. Use YYYY-MM-DD HH:MM".to_owned());
    }
    Ok(DateTimeParts {
        year: numeric[0]
            .parse()
            .map_err(|_| "Invalid format. Use YYYY-MM-DD HH:MM".to_owned())?,
        month: numeric[1]
            .parse()
            .map_err(|_| "Invalid format. Use YYYY-MM-DD HH:MM".to_owned())?,
        day: numeric[2]
            .parse()
            .map_err(|_| "Invalid format. Use YYYY-MM-DD HH:MM".to_owned())?,
        hour: numeric[3]
            .parse()
            .map_err(|_| "Invalid format. Use YYYY-MM-DD HH:MM".to_owned())?,
        minute: numeric[4]
            .parse()
            .map_err(|_| "Invalid format. Use YYYY-MM-DD HH:MM".to_owned())?,
    })
}

pub(crate) fn build_chart(args: CommonChartArgs) -> Result<ChartContext, String> {
    let datetime = parse_datetime(&args.datetime)?;
    let time_index = datetime.time_index();
    // The reference's default profile uses `dayDivide: current`: its late-Zi
    // calculation keeps the civil/lunar day while retaining public timeIndex
    // 12. iztro-rs models late Zi with next-day placement, so its early-Zi
    // variant is the library-backed equivalent for this one profile boundary.
    let calculation_time_index = if args.profile == ZiweiProfile::Sanhe && time_index == 12 {
        0
    } else {
        time_index
    };
    let method = MethodProfile::new(
        args.profile.as_str(),
        args.profile.algorithm(),
        format!("{} compatibility profile", args.profile.label()),
    );

    let (chart, effective_solar_time) = match args.calendar {
        ZiweiCalendar::Solar => {
            let solar_time = datetime.solar_time()?;
            let chart = if args.profile == ZiweiProfile::FeixingSihua {
                // Upstream `yearDivide: exact` uses lunar-typescript's LiChun
                // calendar-day boundary. Resolve the solar date with tyme4rs,
                // then let iztro-rs place every star from those typed facts.
                let lunar_day = solar_time.get_lunar_hour().get_lunar_day();
                let lunar_month = lunar_day.get_lunar_month();
                let birth_year = feixing_year_pillar(&solar_time);
                build_lunar_chart(LunarChartInput {
                    year: lunar_month.get_lunar_year().get_year() as i32,
                    month: lunar_month.get_month() as u8,
                    day: lunar_day.get_day() as u8,
                    is_leap_month: lunar_month.is_leap(),
                    time_index: calculation_time_index,
                    gender: args.gender,
                    birth_year,
                    method: method.clone(),
                })?
            } else {
                let request = SolarChartRequest::builder()
                    .solar_year(datetime.year)
                    .solar_month(
                        SolarMonth::new(datetime.month).map_err(|error| error.to_string())?,
                    )
                    .solar_day(SolarDay::new(datetime.day).map_err(|error| error.to_string())?)
                    .iztro_time_index(calculation_time_index)
                    .map_err(|error| error.to_string())?
                    .gender(args.gender)
                    .fix_leap(true)
                    .year_boundary(args.profile.year_boundary())
                    .method_profile(method.clone())
                    .build()
                    .map_err(|error| error.to_string())?;
                by_solar(request).map_err(|error| error.to_string())?
            };
            (chart, solar_time)
        }
        ZiweiCalendar::Lunar => {
            if !(1..=9_998).contains(&datetime.year) {
                return Err("lunar datetime exceeds tyme4rs safe range (year 1..=9998).".to_owned());
            }
            let lunar_month = if args.is_leap_month {
                -(datetime.month as isize)
            } else {
                datetime.month as isize
            };
            // LunarDay::validate uses the panicking LunarMonth::from_ym helper;
            // validate the parent through its fallible constructor first.
            tyme4rs::tyme::lunar::LunarMonth::new(datetime.year as isize, lunar_month)?;
            let lunar_hour = LunarHour::new(
                datetime.year as isize,
                lunar_month,
                datetime.day as usize,
                datetime.hour as usize,
                datetime.minute as usize,
                0,
            )?;
            let birth_year = StemBranch::from_lunar_year(datetime.year);
            let birth_year = if args.profile == ZiweiProfile::FeixingSihua {
                feixing_year_pillar(&lunar_hour.get_solar_time())
            } else {
                birth_year
            };
            (
                build_lunar_chart(LunarChartInput {
                    year: datetime.year,
                    month: datetime.month,
                    day: datetime.day,
                    is_leap_month: args.is_leap_month,
                    time_index: calculation_time_index,
                    gender: args.gender,
                    birth_year,
                    method: method.clone(),
                })?,
                lunar_hour.get_solar_time(),
            )
        }
    };

    let view = StaticChartViewSnapshot::from_chart(&chart);
    let supplemental_view =
        if view.center.four_pillars.is_none() || view.center.western_zodiac.is_none() {
            Some(solar_fact_view(
                effective_solar_time,
                calculation_time_index,
                args.gender,
                args.profile,
                method,
            )?)
        } else {
            None
        };
    let fact_center = supplemental_view
        .as_ref()
        .map_or(&view.center, |facts| &facts.center);
    let chinese_date = if args.profile == ZiweiProfile::FeixingSihua {
        feixing_four_pillars(&effective_solar_time, time_index)
    } else {
        let four_pillars = fact_center
            .four_pillars
            .as_ref()
            .ok_or_else(|| "Missing natal four-pillar facts.".to_owned())?;
        [
            four_pillars.yearly,
            four_pillars.monthly,
            four_pillars.daily,
            four_pillars.hourly,
        ]
    };
    let solar_label = if args.calendar == ZiweiCalendar::Lunar {
        format!(
            "{}-{}-{}",
            effective_solar_time.get_year(),
            effective_solar_time.get_month(),
            effective_solar_time.get_day()
        )
    } else if view.center.birth_solar_label.is_empty() {
        solar_date_text(effective_solar_time)
    } else {
        view.center.birth_solar_label.clone()
    };
    let lunar_label =
        if args.calendar == ZiweiCalendar::Lunar && !fact_center.birth_lunar_label.is_empty() {
            fact_center.birth_lunar_label.clone()
        } else if view.center.birth_lunar_label.is_empty() {
            let lunar = effective_solar_time.get_lunar_hour().get_lunar_day();
            lunar.to_string()
        } else {
            view.center.birth_lunar_label.clone()
        };
    Ok(ChartContext {
        input_datetime: args.datetime,
        profile: args.profile,
        calendar: args.calendar,
        language: args.language,
        time_index,
        birth_leap_layout_offset: if args.calendar == ZiweiCalendar::Lunar
            && args.is_leap_month
            && datetime.day > 15
        {
            -1
        } else {
            0
        },
        solar_label,
        lunar_label,
        chinese_date,
        western_zodiac: fact_center.western_zodiac,
        chart,
        view,
    })
}

struct LunarChartInput {
    year: i32,
    month: u8,
    day: u8,
    is_leap_month: bool,
    time_index: u8,
    gender: Gender,
    birth_year: StemBranch,
    method: MethodProfile,
}

fn build_lunar_chart(input: LunarChartInput) -> Result<Chart, String> {
    let request = LunarChartRequest::builder()
        .lunar_year(input.year)
        .lunar_month(IztroLunarMonth::new(input.month).map_err(|error| error.to_string())?)
        .lunar_day(IztroLunarDay::new(input.day).map_err(|error| error.to_string())?)
        .iztro_time_index(input.time_index)
        .map_err(|error| error.to_string())?
        .gender(input.gender)
        .birth_year_stem(input.birth_year.stem())
        .birth_year_branch(input.birth_year.branch())
        .is_leap_month(input.is_leap_month)
        .fix_leap(true)
        .method_profile(input.method)
        .build()
        .map_err(|error| error.to_string())?;
    by_lunar(request).map_err(|error| error.to_string())
}

fn feixing_year_pillar(solar_time: &SolarTime) -> StemBranch {
    StemBranch::from_lunar_year(feixing_year_number(solar_time))
}

fn feixing_year_number(solar_time: &SolarTime) -> i32 {
    let year = solar_time.get_year();
    let li_chun = SolarTerm::from_name(year, "立春")
        .get_julian_day()
        .get_solar_time();
    let after_li_chun_day =
        (solar_time.get_month(), solar_time.get_day()) >= (li_chun.get_month(), li_chun.get_day());
    if after_li_chun_day {
        year as i32
    } else {
        year as i32 - 1
    }
}

fn feixing_four_pillars(solar_time: &SolarTime, time_index: u8) -> [StemBranch; 4] {
    // iztro receives only a date and timeIndex. lunar-lite evaluates each
    // double-hour at its representative `HH:30`, not at the caller's minute.
    let representative_hour = if time_index == 0 {
        0
    } else {
        (time_index * 2 - 1) as usize
    };
    let representative = SolarTime::from_ymd_hms(
        solar_time.get_year(),
        solar_time.get_month(),
        solar_time.get_day(),
        representative_hour,
        30,
        0,
    );
    let eight_char = representative.get_lunar_hour().get_eight_char();
    [
        feixing_year_pillar(&representative),
        StemBranch::from_cycle_index(eight_char.get_month().get_index()),
        StemBranch::from_cycle_index(eight_char.get_day().get_index()),
        StemBranch::from_cycle_index(eight_char.get_hour().get_index()),
    ]
}

fn solar_fact_view(
    solar_time: SolarTime,
    time_index: u8,
    gender: Gender,
    profile: ZiweiProfile,
    method: MethodProfile,
) -> Result<StaticChartViewSnapshot, String> {
    let request = SolarChartRequest::builder()
        .solar_year(solar_time.get_year() as i32)
        .solar_month(
            SolarMonth::new(solar_time.get_month() as u8).map_err(|error| error.to_string())?,
        )
        .solar_day(SolarDay::new(solar_time.get_day() as u8).map_err(|error| error.to_string())?)
        .iztro_time_index(time_index)
        .map_err(|error| error.to_string())?
        .gender(gender)
        .fix_leap(true)
        .year_boundary(profile.year_boundary())
        .method_profile(method)
        .build()
        .map_err(|error| error.to_string())?;
    let chart = by_solar(request).map_err(|error| error.to_string())?;
    Ok(StaticChartViewSnapshot::from_chart(&chart))
}

pub(crate) fn build_horoscope(args: HoroscopeArgs) -> Result<HoroscopeContext, String> {
    let birth_datetime = args.birth.datetime.clone();
    let target_datetime = args.target_datetime;
    let target = parse_datetime(&target_datetime)
        .map_err(|_| "Invalid targetDatetime format. Use YYYY-MM-DD HH:MM".to_owned())?;
    validate_horoscope_target_year(target.year as isize)?;
    let target_solar_time = target
        .solar_time()
        .map_err(|_| "Invalid targetDatetime format. Use YYYY-MM-DD HH:MM".to_owned())?;
    let target_time_index = target.time_index();
    let natal = build_chart(args.birth)?;
    let target_time =
        BirthTime::from_iztro_time_index(target_time_index).map_err(|error| error.to_string())?;
    let stack_input = HoroscopeStackInput::new(
        target.year,
        SolarMonth::new(target.month).map_err(|error| error.to_string())?,
        SolarDay::new(target.day).map_err(|error| error.to_string())?,
        target_time,
    )
    .with_nominal_age_boundary(NominalAgeBoundary::NaturalYear);
    let mut horoscope = match build_full_horoscope_chart(natal.chart.clone(), stack_input) {
        Ok(horoscope) => horoscope,
        Err(ChartError::NominalAgeOutsideDecadalFrame { nominal_age })
            if (1..=6).contains(&nominal_age) =>
        {
            build_childhood_horoscope(
                &natal.chart,
                target,
                target_time,
                &target_solar_time,
                nominal_age,
            )?
        }
        Err(error) => return Err(error.to_string()),
    };
    if natal.profile == ZiweiProfile::FeixingSihua {
        horoscope = with_exact_horoscope_layers(horoscope, &target_solar_time, target_time_index)?;
    }
    if natal.birth_leap_layout_offset != 0 {
        horoscope = with_flow_layout_offset(horoscope, natal.birth_leap_layout_offset)?;
    }
    let target_context = horoscope
        .target_context()
        .ok_or_else(|| "Missing target horoscope context.".to_owned())?;
    let solar = target_context.solar_date();
    let lunar = target_context.lunar_date();

    Ok(HoroscopeContext {
        natal,
        birth_datetime,
        target_datetime,
        target_time_index,
        target_solar_time,
        target_solar_label: format!("{}-{}-{}", solar.year(), solar.month(), solar.day()),
        target_lunar_label: chinese_date::lunar_date_label(
            lunar.year(),
            lunar.month(),
            lunar.day(),
            lunar.is_leap_month(),
        ),
        horoscope,
    })
}

/// Builds the upstream 1–6 岁童限 fallback when a child has not entered the
/// first conventional ten-year frame. The period builders remain responsible
/// for every other horoscope layer.
fn build_childhood_horoscope(
    natal: &Chart,
    target: DateTimeParts,
    target_time: BirthTime,
    target_solar_time: &SolarTime,
    nominal_age: u8,
) -> Result<HoroscopeChart, String> {
    let childhood_palace = [
        PalaceName::Life,
        PalaceName::Wealth,
        PalaceName::Health,
        PalaceName::Spouse,
        PalaceName::Spirit,
        PalaceName::Career,
    ][nominal_age as usize - 1];
    let palace = natal
        .required_palace_by_name(childhood_palace)
        .map_err(|error| error.to_string())?;
    let stem_branch =
        StemBranch::try_new(palace.stem(), palace.branch()).map_err(|error| error.to_string())?;
    let decadal_context = TemporalContext::Decadal {
        stem_branch,
        start_age: nominal_age,
    };
    let decadal_flow = build_flow_star_layer(decadal_context).map_err(|error| error.to_string())?;
    let decadal_layout = TemporalPalaceLayout::try_new(
        Scope::Decadal,
        natal
            .palaces()
            .iter()
            .map(|item| {
                TemporalPalaceName::new(
                    item.branch(),
                    item.name().offset(-(palace.name().index() as isize)),
                )
            })
            .collect(),
    )
    .map_err(|error| error.to_string())?;
    let decadal = TemporalLayer::try_new_with_palace_layout(
        Scope::Decadal,
        decadal_context,
        decadal_flow.placements().to_vec(),
        temporal_mutagen_activations(natal, Scope::Decadal, stem_branch.stem()),
        Some(decadal_layout),
    )
    .map_err(|error| error.to_string())?;

    let lunar_day = target_solar_time.get_lunar_hour().get_lunar_day();
    let lunar_month = lunar_day.get_lunar_month();
    let lunar_year = lunar_month.get_lunar_year().get_year() as i32;
    let age = build_age_horoscope_layer(
        natal,
        &build_age_period(natal, nominal_age).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let yearly = build_yearly_horoscope_layer(
        natal,
        &build_yearly_period(lunar_year).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let solar_month = SolarMonth::new(target.month).map_err(|error| error.to_string())?;
    let solar_day = SolarDay::new(target.day).map_err(|error| error.to_string())?;
    let monthly = build_monthly_horoscope_layer(
        natal,
        &build_monthly_period(natal, target.year, solar_month, solar_day, target_time)
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let daily = build_daily_horoscope_layer(
        natal,
        &build_daily_period(natal, target.year, solar_month, solar_day, target_time)
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let hourly = build_hourly_horoscope_layer(
        natal,
        &build_hourly_period(natal, target.year, solar_month, solar_day, target_time)
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let target_context = HoroscopeTargetContext::new(
        HoroscopeSolarDate::new(target.year, target.month, target.day),
        HoroscopeLunarDate::new(
            lunar_year,
            lunar_month.get_month() as u8,
            lunar_day.get_day() as u8,
            lunar_month.is_leap(),
        ),
        target_time.iztro_time_index(),
    );

    Ok(HoroscopeChart::with_layers_and_target_context(
        natal.clone(),
        vec![decadal, age, yearly, monthly, daily, hourly],
        target_context,
    ))
}

fn temporal_mutagen_activations(
    natal: &Chart,
    scope: Scope,
    stem: HeavenlyStem,
) -> Vec<MutagenActivation> {
    natal
        .stars()
        .into_iter()
        .filter_map(|fact| {
            let star = fact.placement().name();
            birth_year_star_mutagen(stem, star)
                .map(|mutagen| MutagenActivation::new(scope, star, fact.palace().branch(), mutagen))
        })
        .collect()
}

/// Adapts iztro-rs's normal-boundary stack to upstream
/// `horoscopeDivide: exact`. Public iztro layer builders still own all star and
/// transformation tables; this adapter supplies the LiChun year and solar-term
/// month facts and rotates only the palace layouts derived from the year branch.
fn with_exact_horoscope_layers(
    horoscope: HoroscopeChart,
    target_solar_time: &SolarTime,
    target_time_index: u8,
) -> Result<HoroscopeChart, String> {
    let original_year = horoscope
        .layers_in_scope(Scope::Yearly)
        .next()
        .ok_or_else(|| "Missing yearly horoscope layer.".to_owned())?;
    let exact_year_period = build_yearly_period(feixing_year_number(target_solar_time))
        .map_err(|error| error.to_string())?;
    let exact_year = build_yearly_horoscope_layer(horoscope.natal(), &exact_year_period)
        .map_err(|error| error.to_string())?;
    let palace_offset = exact_year.context().stem_branch().branch().index() as isize
        - original_year.context().stem_branch().branch().index() as isize;

    let original_month = horoscope
        .layers_in_scope(Scope::Monthly)
        .next()
        .ok_or_else(|| "Missing monthly horoscope layer.".to_owned())?;
    let lunar_month = match *original_month.context() {
        TemporalContext::Monthly { lunar_month, .. } => lunar_month,
        _ => return Err("Invalid monthly horoscope context.".to_owned()),
    };
    let context = TemporalContext::Monthly {
        stem_branch: feixing_four_pillars(target_solar_time, target_time_index)[1],
        lunar_month,
    };
    let flow = build_flow_star_layer(context).map_err(|error| error.to_string())?;
    let activations = temporal_mutagen_activations(
        horoscope.natal(),
        Scope::Monthly,
        context.stem_branch().stem(),
    );
    let exact_month = TemporalLayer::try_new_with_palace_layout(
        Scope::Monthly,
        context,
        flow.placements().to_vec(),
        activations,
        original_month
            .palace_layout()
            .map(|layout| shifted_palace_layout(layout, palace_offset))
            .transpose()?,
    )
    .map_err(|error| error.to_string())?;
    let layers = horoscope
        .layers()
        .iter()
        .map(|layer| match layer.scope() {
            Scope::Yearly => Ok(exact_year.clone()),
            Scope::Monthly => Ok(exact_month.clone()),
            Scope::Daily | Scope::Hourly if palace_offset != 0 => {
                shifted_temporal_layer(layer, palace_offset)
            }
            _ => Ok(layer.clone()),
        })
        .collect::<Result<Vec<_>, String>>()?;
    let target_context = horoscope
        .target_context()
        .cloned()
        .ok_or_else(|| "Missing target horoscope context.".to_owned())?;

    Ok(HoroscopeChart::with_layers_and_target_context(
        horoscope.natal().clone(),
        layers,
        target_context,
    ))
}

/// Applies iztro@2.5.8's late-half natal leap-month correction to the three
/// palace layouts that count backward from the birth month.
fn with_flow_layout_offset(
    horoscope: HoroscopeChart,
    palace_offset: isize,
) -> Result<HoroscopeChart, String> {
    let layers = horoscope
        .layers()
        .iter()
        .map(|layer| match layer.scope() {
            Scope::Monthly | Scope::Daily | Scope::Hourly => {
                shifted_temporal_layer(layer, palace_offset)
            }
            _ => Ok(layer.clone()),
        })
        .collect::<Result<Vec<_>, String>>()?;
    let target_context = horoscope
        .target_context()
        .cloned()
        .ok_or_else(|| "Missing target horoscope context.".to_owned())?;

    Ok(HoroscopeChart::with_layers_and_target_context(
        horoscope.natal().clone(),
        layers,
        target_context,
    ))
}

fn shifted_temporal_layer(
    layer: &TemporalLayer,
    palace_offset: isize,
) -> Result<TemporalLayer, String> {
    let layout = layer
        .palace_layout()
        .map(|layout| shifted_palace_layout(layout, palace_offset))
        .transpose()?;
    TemporalLayer::try_new_with_palace_layout_and_decorative_stars(
        layer.scope(),
        *layer.context(),
        layer.placements().to_vec(),
        layer.activations().to_vec(),
        layout,
        layer.temporal_decorative_stars().to_vec(),
    )
    .map_err(|error| error.to_string())
}

fn shifted_palace_layout(
    layout: &TemporalPalaceLayout,
    palace_offset: isize,
) -> Result<TemporalPalaceLayout, String> {
    let names = layout
        .names()
        .iter()
        .map(|name| {
            TemporalPalaceName::new(name.branch(), name.palace_name().offset(palace_offset))
        })
        .collect();
    TemporalPalaceLayout::try_new(layout.scope(), names).map_err(|error| error.to_string())
}

fn validate_horoscope_target_year(year: isize) -> Result<(), String> {
    // Every runtime overview renders yearly, monthly, daily, and hourly solar
    // ranges. Those inclusive ranges may cross one civil-year boundary, while
    // tyme4rs's infallible range helpers panic outside years 1..=9999.
    if !(2..=9_998).contains(&year) {
        return Err("targetDatetime exceeds tyme4rs safe range (year 2..=9998).".to_owned());
    }
    Ok(())
}

pub(crate) fn scope_solar_range(scope: ZiweiScope, target: &SolarTime) -> Result<String, String> {
    validate_horoscope_target_year(target.get_year())?;
    let hour = target.get_sixty_cycle_hour();
    let day = hour.get_sixty_cycle_day();
    let month = day.get_sixty_cycle_month();
    Ok(match scope {
        ZiweiScope::Yearly => {
            let start = sixty_cycle_month_start(&month.get_sixty_cycle_year().get_first_month());
            let end =
                sixty_cycle_month_start(&month.get_sixty_cycle_year().next(1).get_first_month())
                    .next(-1);
            solar_range_text(start, end)
        }
        ZiweiScope::Monthly => {
            let start = sixty_cycle_month_start(&month);
            let end = sixty_cycle_month_start(&month.next(1)).next(-1);
            solar_range_text(start, end)
        }
        ZiweiScope::Daily => {
            let hours = day.get_hours();
            match (hours.first(), hours.last()) {
                (Some(first), Some(last)) => {
                    solar_range_text(first.get_solar_time(), last.get_solar_time().next(7_199))
                }
                _ => "-".to_owned(),
            }
        }
        ZiweiScope::Hourly => {
            let solar_time = hour.get_solar_time();
            let start = start_of_sixty_cycle_hour(&solar_time);
            solar_range_text(start, start.next(7_199))
        }
        ZiweiScope::Decadal | ZiweiScope::Age => "-".to_owned(),
    })
}

fn sixty_cycle_month_start(month: &SixtyCycleMonth) -> SolarTime {
    SolarTerm::from_index(
        month.get_sixty_cycle_year().get_year(),
        3 + month.get_index_in_year() as isize * 2,
    )
    .get_julian_day()
    .get_solar_time()
}

fn start_of_sixty_cycle_hour(solar_time: &SolarTime) -> SolarTime {
    let hour = solar_time.get_hour();
    if hour == 0 {
        return SolarTime::from_ymd_hms(
            solar_time.get_year(),
            solar_time.get_month(),
            solar_time.get_day(),
            0,
            0,
            0,
        )
        .next(-3_600);
    }
    let start_hour = if hour == 23 {
        23
    } else {
        hour.div_ceil(2) * 2 - 1
    };
    SolarTime::from_ymd_hms(
        solar_time.get_year(),
        solar_time.get_month(),
        solar_time.get_day(),
        start_hour,
        0,
        0,
    )
}

fn solar_date_text(time: SolarTime) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        time.get_year(),
        time.get_month(),
        time.get_day()
    )
}

fn solar_range_text(start: SolarTime, end: SolarTime) -> String {
    format!("{} 至 {}", solar_time_text(start), solar_time_text(end))
}

fn solar_time_text(time: SolarTime) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        time.get_year(),
        time.get_month(),
        time.get_day(),
        time.get_hour(),
        time.get_minute(),
        time.get_second()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_index_matches_upstream_edges() {
        let mut value = parse_datetime("2024-01-15 00:30").unwrap();
        assert_eq!(value.time_index(), 0);
        value.hour = 18;
        assert_eq!(value.time_index(), 9);
        value.hour = 23;
        assert_eq!(value.time_index(), 12);
    }

    #[test]
    fn rejects_non_contract_datetime_shape() {
        assert!(parse_datetime("2024-1-15 08:30").is_err());
        assert!(parse_datetime(" 2024-01-15 08:30").is_err());
        assert!(parse_datetime("2024-01-15T08:30").is_err());
    }
}

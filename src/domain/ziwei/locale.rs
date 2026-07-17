//! Pure, table-driven localization for Zi Wei Dou Shu presentation values.
//!
//! The translation rows are copied verbatim from the locale modules bundled
//! with iztro@2.5.8. Rust enum identities and canonical Simplified-Chinese
//! labels come from iztro 0.9.0, so calculation remains language-neutral.
//! Unknown values deliberately fall back to the caller's canonical text;
//! translations not published by upstream are never invented here.
//!
//! Upstream source: <https://registry.npmjs.org/iztro/-/iztro-2.5.8.tgz>
//! License: MIT, reproduced below as required for this substantial data copy.
//!
//! MIT License
//!
//! Copyright (c) 2023 All Contributors
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.

use iztro::core::labels::{chinese_date, zh_cn};
use iztro::{
    Brightness, EarthlyBranch, FiveElementBureau, HeavenlyStem, Mutagen, PalaceName, StarName,
    StemBranch, WesternZodiac,
};
#[cfg(test)]
use iztro::{Gender, Scope};

/// Languages exposed by the reference MCP schema.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Language {
    /// Simplified Chinese.
    #[default]
    ZhCn,
    /// Traditional Chinese.
    ZhTw,
    /// American English.
    EnUs,
    /// Japanese.
    JaJp,
    /// Korean.
    KoKr,
    /// Vietnamese.
    ViVn,
}

impl Language {
    /// Parses the exact BCP-47-style values accepted by the MCP input schema.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "zh-CN" => Ok(Self::ZhCn),
            "zh-TW" => Ok(Self::ZhTw),
            "en-US" => Ok(Self::EnUs),
            "ja-JP" => Ok(Self::JaJp),
            "ko-KR" => Ok(Self::KoKr),
            "vi-VN" => Ok(Self::ViVn),
            _ => Err(
                "Unsupported language. Use zh-CN, zh-TW, en-US, ja-JP, ko-KR, or vi-VN.".to_owned(),
            ),
        }
    }

    /// Returns the exact language code used by the public MCP contract.
    #[cfg(test)]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::ZhTw => "zh-TW",
            Self::EnUs => "en-US",
            Self::JaJp => "ja-JP",
            Self::KoKr => "ko-KR",
            Self::ViVn => "vi-VN",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::ZhCn => 0,
            Self::ZhTw => 1,
            Self::EnUs => 2,
            Self::JaJp => 3,
            Self::KoKr => 4,
            Self::ViVn => 5,
        }
    }

    /// Looks up a published upstream translation.
    ///
    /// Returns None for values outside the selected upstream namespace.
    pub fn lookup(self, kind: LocaleKind, canonical_zh: &str) -> Option<&'static str> {
        lookup_row(kind, canonical_zh).map(|row| row.values[self.index()])
    }

    /// Translates a canonical Simplified-Chinese value, preserving unknown input.
    pub fn translate(self, kind: LocaleKind, canonical_zh: &str) -> &str {
        self.lookup(kind, canonical_zh).unwrap_or(canonical_zh)
    }

    /// Localizes an iztro palace enum.
    pub fn palace(self, value: PalaceName) -> &'static str {
        self.translate(LocaleKind::Palace, zh_cn::palace_name_zh(value))
    }

    /// Localizes an iztro star enum, including decorative and flow stars.
    pub fn star(self, value: StarName) -> &'static str {
        self.translate(LocaleKind::Star, zh_cn::star_name_zh(value))
    }

    /// Localizes a decorative/flow star through iztro's shared star namespace.
    pub fn decorative_star(self, value: StarName) -> &'static str {
        self.translate(LocaleKind::Decorative, zh_cn::star_name_zh(value))
    }

    /// Localizes a calculated brightness. Unknown brightness remains empty.
    pub fn brightness(self, value: Brightness) -> &'static str {
        self.translate(LocaleKind::Brightness, zh_cn::brightness_zh(value))
    }

    /// Localizes one of the four transformations.
    pub fn mutagen(self, value: Mutagen) -> &'static str {
        self.translate(LocaleKind::Mutagen, zh_cn::mutagen_zh(value))
    }

    /// Localizes a Heavenly Stem.
    pub fn heavenly_stem(self, value: HeavenlyStem) -> &'static str {
        self.translate(LocaleKind::Stem, zh_cn::heavenly_stem_zh(value))
    }

    /// Localizes an Earthly Branch.
    pub fn earthly_branch(self, value: EarthlyBranch) -> &'static str {
        self.translate(LocaleKind::Branch, zh_cn::earthly_branch_zh(value))
    }

    /// Localizes a stem-branch pair without changing its canonical order.
    pub fn stem_branch(self, value: StemBranch) -> String {
        format!(
            "{}{}",
            self.heavenly_stem(value.stem()),
            self.earthly_branch(value.branch())
        )
    }

    /// Localizes a Five Element Bureau.
    pub fn five_element_bureau(self, value: FiveElementBureau) -> &'static str {
        self.translate(
            LocaleKind::FiveElement,
            zh_cn::five_element_bureau_zh(value),
        )
    }

    /// Localizes the gender marker used by iztro.
    #[cfg(test)]
    pub fn gender(self, value: Gender) -> &'static str {
        let canonical = match value {
            Gender::Male => "男",
            Gender::Female => "女",
        };
        self.translate(LocaleKind::Gender, canonical)
    }

    /// Localizes the Chinese zodiac animal associated with a branch.
    pub fn zodiac_animal(self, value: EarthlyBranch) -> &'static str {
        self.translate(LocaleKind::Zodiac, zh_cn::zodiac_animal_zh(value))
    }

    /// Localizes a Western zodiac sign.
    pub fn western_zodiac(self, value: WesternZodiac) -> &'static str {
        self.translate(LocaleKind::Sign, chinese_date::western_zodiac_zh(value))
    }

    /// Localizes a horoscope scope when iztro@2.5.8 publishes that label.
    ///
    /// Natal (本命) has no upstream locale key and therefore falls back to
    /// its canonical Chinese label.
    #[cfg(test)]
    pub fn scope(self, value: Scope) -> &'static str {
        self.translate(LocaleKind::Scope, zh_cn::scope_zh(value))
    }

    /// Localizes any canonical value from iztro's common.json table.
    #[cfg(test)]
    pub fn common(self, canonical_zh: &str) -> &str {
        self.translate(LocaleKind::Common, canonical_zh)
    }
}

impl TryFrom<&str> for Language {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

/// Translation namespaces published by iztro@2.5.8.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LocaleKind {
    Palace,
    Star,
    Brightness,
    Mutagen,
    Stem,
    Branch,
    FiveElement,
    #[cfg(test)]
    Gender,
    Zodiac,
    Sign,
    #[cfg(test)]
    Scope,
    /// Alias for iztro's star namespace, where decorative names are stored.
    Decorative,
    /// Scope, zodiac, hour-name, and Western-sign values from common.json.
    #[cfg(test)]
    Common,
}

/// Returns the number of published canonical values in a locale namespace.
#[cfg(test)]
pub const fn entry_count(kind: LocaleKind) -> usize {
    match kind {
        LocaleKind::Palace => PALACE.len(),
        LocaleKind::Star | LocaleKind::Decorative => STAR.len(),
        LocaleKind::Brightness => BRIGHTNESS.len(),
        LocaleKind::Mutagen => MUTAGEN.len(),
        LocaleKind::Stem => STEM.len(),
        LocaleKind::Branch => BRANCH.len(),
        LocaleKind::FiveElement => FIVE_ELEMENT.len(),
        #[cfg(test)]
        LocaleKind::Gender => GENDER.len(),
        LocaleKind::Zodiac => ZODIAC.len(),
        LocaleKind::Sign => SIGN.len(),
        #[cfg(test)]
        LocaleKind::Scope => SCOPE.len(),
        #[cfg(test)]
        LocaleKind::Common => COMMON.len(),
    }
}

#[derive(Clone, Copy)]
struct Row {
    values: [&'static str; 6],
}

impl Row {
    const fn new(
        zh_cn: &'static str,
        zh_tw: &'static str,
        en_us: &'static str,
        ja_jp: &'static str,
        ko_kr: &'static str,
        vi_vn: &'static str,
    ) -> Self {
        Self {
            values: [zh_cn, zh_tw, en_us, ja_jp, ko_kr, vi_vn],
        }
    }

    const fn canonical(self) -> &'static str {
        self.values[0]
    }
}

fn lookup_row(kind: LocaleKind, canonical_zh: &str) -> Option<&'static Row> {
    let rows = match kind {
        LocaleKind::Palace => PALACE.as_slice(),
        LocaleKind::Star | LocaleKind::Decorative => STAR.as_slice(),
        LocaleKind::Brightness => BRIGHTNESS.as_slice(),
        LocaleKind::Mutagen => MUTAGEN.as_slice(),
        LocaleKind::Stem => STEM.as_slice(),
        LocaleKind::Branch => BRANCH.as_slice(),
        LocaleKind::FiveElement => FIVE_ELEMENT.as_slice(),
        #[cfg(test)]
        LocaleKind::Gender => GENDER.as_slice(),
        LocaleKind::Zodiac => ZODIAC.as_slice(),
        LocaleKind::Sign => SIGN.as_slice(),
        #[cfg(test)]
        LocaleKind::Scope => SCOPE.as_slice(),
        #[cfg(test)]
        LocaleKind::Common => COMMON.as_slice(),
    };
    rows.iter().find(|row| row.canonical() == canonical_zh)
}

const PALACE: [Row; 14] = [
    Row::new("命宫", "命宮", "soul", "命宮", "명궁", "Mệnh"),
    Row::new("身宫", "身宮", "body", "身宮", "신궁", "Thân"),
    Row::new("兄弟", "兄弟", "siblings", "兄弟", "형제", "Huynh Đệ"),
    Row::new("夫妻", "夫妻", "spouse", "夫妻", "부처", "Phu Thê"),
    Row::new("子女", "子女", "children", "子女", "자녀", "Tử Nữ"),
    Row::new("财帛", "財帛", "wealth", "財帛", "재백", "Tài Bạch"),
    Row::new("疾厄", "疾厄", "health", "疾厄", "질액", "Tật Ách"),
    Row::new("迁移", "遷移", "surface", "遷移", "천이", "Thiên Di"),
    Row::new("仆役", "僕役", "friends", "僕役", "노복", "Nô Bộc"),
    Row::new("官禄", "官祿", "career", "官祿", "관록", "Quan Lộc"),
    Row::new("田宅", "田宅", "property", "田宅", "전택", "Điền Trạch"),
    Row::new("福德", "福德", "spirit", "福德", "복덕", "Phúc Đức"),
    Row::new("父母", "父母", "parents", "父母", "부모", "Phụ Mẫu"),
    Row::new("来因", "来因", "origin", "来因", "라인", "Lai Nhân"),
];

const STAR: [Row; 162] = [
    Row::new("紫微", "紫微", "emperor", "紫微", "자미", "Tử Vi"),
    Row::new("天机", "天機", "advisor", "天機", "천기", "Thiên Cơ"),
    Row::new("太阳", "太陽", "sun", "太陽", "태양", "Thái Dương"),
    Row::new("武曲", "武曲", "general", "武曲", "무곡", "Vũ Khúc"),
    Row::new("天同", "天同", "fortunate", "天同", "천동", "Thiên Đồng"),
    Row::new("廉贞", "廉貞", "judge", "廉貞", "염정", "Liêm Trinh"),
    Row::new("天府", "天府", "empress", "天府", "천부", "Thiên Phủ"),
    Row::new("太阴", "太陰", "moon", "太陰", "태음", "Thái Âm"),
    Row::new("贪狼", "貪狼", "wolf", "貪狼", "탐랑", "Tham Lang"),
    Row::new("巨门", "巨門", "advocator", "巨門", "거문", "Cự Môn"),
    Row::new("天相", "天相", "minister", "天相", "천상", "Thiên Tướng"),
    Row::new("天梁", "天梁", "sage", "天梁", "천량", "Thiên Lương"),
    Row::new("七杀", "七殺", "marshal", "七殺", "칠살", "Thất Sát"),
    Row::new("破军", "破軍", "rebel", "破軍", "파군", "Phá Quân"),
    Row::new("左辅", "左輔", "officer", "左輔", "좌보", "Tả Phù"),
    Row::new("右弼", "右弼", "helper", "右弼", "우필", "Hữu Bật"),
    Row::new("文昌", "文昌", "scholar", "文昌", "문창", "Văn Xương"),
    Row::new("文曲", "文曲", "artist", "文曲", "문곡", "Văn Khúc"),
    Row::new("禄存", "祿存", "money", "祿存", "록존", "Lộc Tồn"),
    Row::new("天马", "天馬", "horse", "天馬", "천마", "Thiên Mã"),
    Row::new("擎羊", "擎羊", "driven", "擎羊", "경양", "Kình Dương"),
    Row::new("陀罗", "陀羅", "tangled", "陀羅", "타라", "Đà La"),
    Row::new("火星", "火星", "impulsive", "火星", "화성", "Hỏa Tinh"),
    Row::new("铃星", "鈴星", "spark", "鈴星", "령성", "Linh Tinh"),
    Row::new("天魁", "天魁", "assistant", "天魁", "천괴", "Thiên Khôi"),
    Row::new("天钺", "天鉞", "aide", "天鉞", "천월", "Thiên Việt"),
    Row::new("地空", "地空", "ideologue", "地空", "지공", "Địa Không"),
    Row::new("地劫", "地劫", "fickle", "地劫", "지겁", "Địa Kiếp"),
    Row::new("劫杀", "劫殺", "murder", "劫殺", "겁살", "Kiếp Sát"),
    Row::new("天空", "天空", "utopian", "天空", "천공", "Thiên Không"),
    Row::new("天刑", "天刑", "serious", "天刑", "천형", "Thiên Hình"),
    Row::new("天姚", "天姚", "social", "天姚", "천요", "Thiên Diêu"),
    Row::new("解神", "解神", "considery", "解神", "해신", "Giải Thần"),
    Row::new("阴煞", "陰煞", "gloomy", "陰煞", "음살", "Âm Sát"),
    Row::new("天喜", "天喜", "cheerful", "天喜", "천희", "Thiên Hỷ"),
    Row::new("天官", "天官", "solemn", "天官", "천관", "Thiên Quan"),
    Row::new("天福", "天福", "lucky", "天福", "천복", "Thiên Phúc"),
    Row::new("天哭", "天哭", "upset", "天哭", "천곡", "Thiên Khốc"),
    Row::new("天虚", "天虛", "frail", "天虛", "천허", "Thiên Hư"),
    Row::new("龙池", "龍池", "talented", "龍池", "용지", "Long Trì"),
    Row::new("凤阁", "鳳閣", "refined", "鳳閣", "봉각", "Phụng Các"),
    Row::new("红鸾", "紅鸞", "attractive", "紅鸞", "홍란", "Hồng Loan"),
    Row::new("孤辰", "孤辰", "alone", "孤辰", "고진", "Cô Thần"),
    Row::new("寡宿", "寡宿", "lonely", "寡宿", "과숙", "Quả Tú"),
    Row::new("蜚廉", "蜚廉", "instigated", "蜚廉", "비렴", "Phi Liêm"),
    Row::new("破碎", "破碎", "broken", "破碎", "파쇄", "Phá Toái"),
    Row::new("台辅", "台輔", "honorable", "台輔", "태보", "Đài Phụ"),
    Row::new("封诰", "封誥", "awarded", "封誥", "봉고", "Phong Cáo"),
    Row::new("天巫", "天巫", "psychic", "天巫", "천무", "Thiên Vu"),
    Row::new("天月", "天月", "sickly", "天月", "천월", "Thiên Nguyệt"),
    Row::new("三台", "三台", "senior", "三台", "삼태", "Tam Thai"),
    Row::new("八座", "八座", "dignified", "八座", "팔좌", "Bát Tọa"),
    Row::new("恩光", "恩光", "grateful", "恩光", "은광", "Ân Quang"),
    Row::new("天贵", "天貴", "noble", "天貴", "천귀", "Thiên Quý"),
    Row::new("天才", "天才", "gifted", "天才", "천재", "Thiên Tài"),
    Row::new("天寿", "天壽", "ageless", "天壽", "천수", "Thiên Thọ"),
    Row::new("截空", "截空", "interrupted", "截空", "절중", "Triệt Không"),
    Row::new("旬中", "旬中", "meditative", "旬中", "순중", "Tuần Trung"),
    Row::new("旬空", "旬空", "fancied", "旬空", "순공", "Tuần Không"),
    Row::new("空亡", "空亡", "bottomless", "空亡", "공망", "Không Vong"),
    Row::new("截路", "截路", "intercepted", "截路", "절로", "Triệt Lộ"),
    Row::new("月德", "月德", "peaceful", "月德", "월덕", "Nguyệt Đức"),
    Row::new("天伤", "天傷", "wounded", "天傷", "천상", "Thiên Thương"),
    Row::new("天使", "天使", "heaven", "天使", "천사", "Thiên Sứ"),
    Row::new("天厨", "天廚", "gourmet", "天廚", "천주", "Thiên Trù"),
    Row::new("长生", "長生", "born", "長生", "장생", "Trường Sinh"),
    Row::new("沐浴", "沐浴", "infancy", "沐浴", "목욕", "Mục Dục"),
    Row::new("冠带", "冠帶", "adolescence", "冠帶", "관대", "Quan Đới"),
    Row::new("临官", "臨官", "adulthood", "臨官", "임관", "Lâm Quan"),
    Row::new("帝旺", "帝旺", "prime", "帝旺", "제왕", "Đế Vượng"),
    Row::new("衰", "衰", "weak", "衰", "쇠", "Suy"),
    Row::new("病", "病", "sick", "病", "병", "Bệnh"),
    Row::new("死", "死", "dead", "死", "사", "Tử"),
    Row::new("墓", "墓", "buried", "墓", "묘", "Mộ"),
    Row::new("绝", "絕", "dissipated", "絕", "절", "Tuyệt"),
    Row::new("胎", "胎", "embryo", "胎", "태", "Thai"),
    Row::new("养", "養", "molding", "養", "양", "Dưỡng"),
    Row::new("博士", "博士", "doctor", "博士", "박사", "Bác Sỹ"),
    Row::new("力士", "力士", "sumo", "力士", "역사", "Lực Sỹ"),
    Row::new("青龙", "青龍", "dragon", "青龍", "청룡", "Thanh Long"),
    Row::new("小耗", "小耗", "consumer", "小耗", "소모", "Tiểu Hao"),
    Row::new("将军", "將軍", "general", "將軍", "장군", "Tướng Quân"),
    Row::new("奏书", "奏書", "book", "奏書", "주서", "Tấu Thư"),
    Row::new("飞廉", "飛廉", "gossip", "飛廉", "비렴", "Phi Liêm"),
    Row::new("喜神", "喜神", "happiness", "喜神", "희신", "Hỷ Thần"),
    Row::new("病符", "病符", "illness", "病符", "병부", "Bệnh Phù"),
    Row::new("大耗", "大耗", "wastrel", "大耗", "대모", "Đại Hao"),
    Row::new("岁破", "歲破", "wastrel", "歲破", "태파", "Tuế Phá"),
    Row::new("伏兵", "伏兵", "ambush", "伏兵", "복병", "Phục Binh"),
    Row::new("官府", "官府", "government", "官府", "관부", "Quan Phủ"),
    Row::new("岁建", "歲建", "initial", "歲建", "태세", "Tuế Kiện"),
    Row::new("晦气", "晦氣", "unlucky", "晦氣", "회기", "Hối Khí"),
    Row::new("丧门", "喪門", "downcast", "喪門", "상문", "Tang Môn"),
    Row::new("贯索", "貫索", "tied", "貫索", "관색", "Quán Tác"),
    Row::new("官符", "官符", "official", "官符", "관부", "Quan Phù"),
    Row::new("龙德", "龍德", "virtuous", "龍德", "용덕", "Long Đức"),
    Row::new("白虎", "白虎", "sinister", "白虎", "백호", "Bạch Hổ"),
    Row::new("天德", "天德", "blessed", "天德", "복덕", "Thiên Đức"),
    Row::new("吊客", "弔客", "sorrowing", "弔客", "조객", "Điếu Khách"),
    Row::new("将星", "將星", "capable", "將星", "장성", "Tướng Tinh"),
    Row::new("攀鞍", "攀鞍", "admired", "攀鞍", "반안", "Phan Án"),
    Row::new("岁驿", "歲驛", "varied", "歲驛", "세역", "Tuế Dịch"),
    Row::new("息神", "息神", "listless", "息神", "식신", "Tức Thần"),
    Row::new("华盖", "華蓋", "religious", "華蓋", "화개", "Hoa Cái"),
    Row::new("劫煞", "劫煞", "robbed", "劫煞", "겁살", "Kiếp Sát"),
    Row::new("灾煞", "災煞", "disastery", "災煞", "재살", "Tai Sát"),
    Row::new("天煞", "天煞", "condemned", "天煞", "천살", "Thiên Sát"),
    Row::new("指背", "指背", "insidious", "指背", "지배", "Chỉ Bối"),
    Row::new("咸池", "咸池", "passionate", "咸池", "함지", "Hàm Trì"),
    Row::new("月煞", "月煞", "hapless", "月煞", "월살", "Nguyệt Sát"),
    Row::new("亡神", "亡神", "perished", "亡神", "망신", "Vong Thần"),
    Row::new(
        "运魁",
        "運魁",
        "assistant(D)",
        "限の魁",
        "천괴(십년)",
        "Vận Khôi",
    ),
    Row::new(
        "运钺",
        "運鉞",
        "aide(D)",
        "限の钺",
        "천월(십년)",
        "Vận Việt",
    ),
    Row::new(
        "运昌",
        "運昌",
        "scholar(D)",
        "限の昌",
        "문창(십년)",
        "Vận Xương",
    ),
    Row::new(
        "运曲",
        "運曲",
        "artist(D)",
        "限の曲",
        "문곡(십년)",
        "Vận Khúc",
    ),
    Row::new(
        "运鸾",
        "運鸞",
        "attractive(D)",
        "限の鸾",
        "홍란(십년)",
        "Vận Loan",
    ),
    Row::new(
        "运喜",
        "運喜",
        "cheerful(D)",
        "限の喜",
        "천희(십년)",
        "Vận Hỷ",
    ),
    Row::new(
        "运禄",
        "運祿",
        "money(D)",
        "限の祿",
        "록존(십년)",
        "Vận Lộc",
    ),
    Row::new(
        "运羊",
        "運羊",
        "driven(D)",
        "限の羊",
        "경양(십년)",
        "Vận Dương",
    ),
    Row::new(
        "运陀",
        "運陀",
        "tangled(D)",
        "限の陀",
        "타라(십년)",
        "Vận Đà",
    ),
    Row::new("运马", "運馬", "horse(D)", "限の馬", "천마(십년)", "Vận Mã"),
    Row::new(
        "流魁",
        "流魁",
        "assistant(Y)",
        "年の魁",
        "천괴(년)",
        "Lưu Khôi",
    ),
    Row::new("流钺", "流鉞", "aide(Y)", "年の钺", "천월(년)", "Lưu Việt"),
    Row::new(
        "流昌",
        "流昌",
        "scholar(Y)",
        "年の昌",
        "문창(년)",
        "Lưu Xương",
    ),
    Row::new(
        "流曲",
        "流曲",
        "artist(Y)",
        "年の曲",
        "문곡(년)",
        "Lưu Khúc",
    ),
    Row::new(
        "流鸾",
        "流鸞",
        "attractive(Y)",
        "年の鸾",
        "홍란(년)",
        "Lưu Loan",
    ),
    Row::new(
        "流喜",
        "流喜",
        "cheerful(Y)",
        "年の喜",
        "천희(년)",
        "Lưu Hỷ",
    ),
    Row::new("流禄", "流祿", "money(Y)", "年の祿", "록존(년)", "Lưu Lộc"),
    Row::new(
        "流羊",
        "流羊",
        "driven(Y)",
        "年の羊",
        "경양(년)",
        "Lưu Dương",
    ),
    Row::new("流陀", "流陀", "tangled(Y)", "年の陀", "타라(년)", "Lưu Đà"),
    Row::new("流马", "流馬", "horse(Y)", "年の馬", "천마(년)", "Lưu Mã"),
    Row::new(
        "年解",
        "年解",
        "considery(Y)",
        "年の解",
        "해신(년)",
        "Niên Giải",
    ),
    Row::new(
        "月魁",
        "月魁",
        "assistant(M)",
        "月の魁",
        "천괴(월)",
        "Thiên Khôi(M)",
    ),
    Row::new(
        "月钺",
        "月鉞",
        "aide(M)",
        "月の钺",
        "천월(월)",
        "Thiên Nguyệt(M)",
    ),
    Row::new(
        "月昌",
        "月昌",
        "scholar(M)",
        "月の昌",
        "문창(월)",
        "Văn Xương(M)",
    ),
    Row::new(
        "月曲",
        "月曲",
        "artist(M)",
        "月の曲",
        "문곡(월)",
        "Văn Khúc(M)",
    ),
    Row::new(
        "月鸾",
        "月鸞",
        "attractive(M)",
        "月の鸾",
        "홍란(월)",
        "Hồng Loan(M)",
    ),
    Row::new(
        "月喜",
        "月喜",
        "cheerful(M)",
        "月の喜",
        "천희(월)",
        "Thiên Hỷ(M)",
    ),
    Row::new(
        "月禄",
        "月祿",
        "money(M)",
        "月の祿",
        "록존(월)",
        "Lộc Tồn(M)",
    ),
    Row::new(
        "月羊",
        "月羊",
        "driven(M)",
        "月の羊",
        "경양(월)",
        "Kình Dương(M)",
    ),
    Row::new(
        "月陀",
        "月陀",
        "tangled(M)",
        "月の陀",
        "타라(월)",
        "Đà La(M)",
    ),
    Row::new(
        "月马",
        "月馬",
        "horse(M)",
        "月の馬",
        "천마(월)",
        "Thiên Mã(M)",
    ),
    Row::new(
        "日魁",
        "日魁",
        "assistant(d)",
        "日の魁",
        "천괴(일)",
        "Thiên Khôi(d)",
    ),
    Row::new(
        "日钺",
        "日鉞",
        "aide(d)",
        "日の钺",
        "천월(일)",
        "Thiên Nguyệt(d)",
    ),
    Row::new(
        "日昌",
        "日昌",
        "scholar(d)",
        "日の昌",
        "문창(일)",
        "Văn Xương(d)",
    ),
    Row::new(
        "日曲",
        "日曲",
        "artist(d)",
        "日の曲",
        "문곡(일)",
        "Văn Khúc(d)",
    ),
    Row::new(
        "日鸾",
        "日鸞",
        "attractive(d)",
        "日の鸾",
        "홍란(일)",
        "Hồng Loan(d)",
    ),
    Row::new(
        "日喜",
        "日喜",
        "cheerful(d)",
        "日の喜",
        "천희(일)",
        "Thiên Hỷ(d)",
    ),
    Row::new(
        "日禄",
        "日祿",
        "money(d)",
        "日の祿",
        "록존(일)",
        "Lộc Tồn(d)",
    ),
    Row::new(
        "日羊",
        "日羊",
        "driven(d)",
        "日の羊",
        "경양(일)",
        "Kình Dương(d)",
    ),
    Row::new(
        "日陀",
        "日陀",
        "tangled(d)",
        "日の陀",
        "타라(일)",
        "Đà La(d)",
    ),
    Row::new(
        "日马",
        "日馬",
        "horse(d)",
        "日の馬",
        "천마(일)",
        "Thiên Mã(d)",
    ),
    Row::new(
        "时魁",
        "時魁",
        "assistant(H)",
        "時の魁",
        "천괴(시)",
        "Thiên Khôi(H)",
    ),
    Row::new(
        "时钺",
        "時鉞",
        "aide(H)",
        "時の钺",
        "천월(시)",
        "Thiên Nguyệt(H)",
    ),
    Row::new(
        "时昌",
        "時昌",
        "scholar(H)",
        "時の昌",
        "문창(시)",
        "Văn Xương(H)",
    ),
    Row::new(
        "时曲",
        "時曲",
        "artist(H)",
        "時の曲",
        "문곡(시)",
        "Văn Khúc(H)",
    ),
    Row::new(
        "时鸾",
        "時鸞",
        "attractive(H)",
        "時の鸾",
        "홍란(시)",
        "Hồng Loan(H)",
    ),
    Row::new(
        "时喜",
        "時喜",
        "cheerful(H)",
        "時の喜",
        "천희(시)",
        "Thiên Hỷ(H)",
    ),
    Row::new(
        "时禄",
        "時祿",
        "money(H)",
        "時の祿",
        "록존(시)",
        "Lộc Tồn(H)",
    ),
    Row::new(
        "时羊",
        "時羊",
        "driven(H)",
        "時の羊",
        "경양(시)",
        "Kình Dương(H)",
    ),
    Row::new(
        "时陀",
        "時陀",
        "tangled(H)",
        "時の陀",
        "타라(시)",
        "Đà La(H)",
    ),
    Row::new(
        "时马",
        "時馬",
        "horse(H)",
        "時の馬",
        "천마(시)",
        "Thiên Mã(H)",
    ),
];

const BRIGHTNESS: [Row; 7] = [
    Row::new("庙", "廟", "[+3]", "廟", "[+3]", "Miếu"),
    Row::new("旺", "旺", "[+2]", "旺", "[+2]", "Vượng"),
    Row::new("得", "得", "[+1]", "得", "[+1]", "Đắc"),
    Row::new("利", "利", "[0]", "利", "[0]", "Lợi"),
    Row::new("平", "平", "[-1]", "平", "[-1]", "Bình"),
    Row::new("不", "不", "[-2]", "不", "[-2]", "Bất"),
    Row::new("陷", "陷", "[-3]", "陷", "[-3]", "Hạn"),
];

const MUTAGEN: [Row; 4] = [
    Row::new("禄", "祿", "A", "祿", "록", "Lộc"),
    Row::new("权", "權", "B", "權", "권", "Quyền"),
    Row::new("科", "科", "C", "科", "과", "Khoa"),
    Row::new("忌", "忌", "D", "忌", "기", "Kỵ"),
];

const FIVE_ELEMENT: [Row; 5] = [
    Row::new(
        "水二局",
        "水二局",
        "water 2nd",
        "水の二局",
        "수이국",
        "Thủy Nhị Cục",
    ),
    Row::new(
        "木三局",
        "木三局",
        "wood 3rd",
        "木の三局",
        "목삼국",
        "Mộc Tam Cục",
    ),
    Row::new(
        "金四局",
        "金四局",
        "metal 4th",
        "金の四局",
        "금사국",
        "Kim Tứ Cục",
    ),
    Row::new(
        "土五局",
        "土五局",
        "earth 5th",
        "土の五局",
        "토오국",
        "Thổ Ngũ Cục",
    ),
    Row::new(
        "火六局",
        "火六局",
        "fire 6th",
        "火の六局",
        "화육국",
        "Hỏa Lục Cục",
    ),
];

#[cfg(test)]
const GENDER: [Row; 2] = [
    Row::new("男", "男", "male", "男", "남성", "Nam"),
    Row::new("女", "女", "female", "女", "여자", "Nữ"),
];

const STEM: [Row; 10] = [
    Row::new("甲", "甲", "jia", "甲", "갑", "Giáp"),
    Row::new("乙", "乙", "yi", "乙", "을", "Ất"),
    Row::new("丙", "丙", "bing", "丙", "병", "Bính"),
    Row::new("丁", "丁", "ding", "丁", "정", "Đinh"),
    Row::new("戊", "戊", "wu", "戊", "무", "Mậu"),
    Row::new("己", "己", "ji", "己", "기", "Kỷ"),
    Row::new("庚", "庚", "geng", "庚", "경", "Canh"),
    Row::new("辛", "辛", "xin", "辛", "신", "Tân"),
    Row::new("壬", "壬", "ren", "壬", "임", "Nhâm"),
    Row::new("癸", "癸", "gui", "癸", "계", "Quý"),
];

const BRANCH: [Row; 12] = [
    Row::new("子", "子", "zi", "子", "자", "Tý"),
    Row::new("丑", "丑", "chou", "丑", "축", "Sửu"),
    Row::new("寅", "寅", "yin", "寅", "인", "Dần"),
    Row::new("卯", "卯", "mao", "卯", "묘", "Mão"),
    Row::new("辰", "辰", "chen", "辰", "진", "Thìn"),
    Row::new("巳", "巳", "si", "巳", "사", "Tỵ"),
    Row::new("午", "午", "woo", "午", "오", "Ngọ"),
    Row::new("未", "未", "wei", "未", "미", "Mùi"),
    Row::new("申", "申", "shen", "申", "신", "Thân"),
    Row::new("酉", "酉", "you", "酉", "유", "Dậu"),
    Row::new("戌", "戌", "xu", "戌", "술", "Tuất"),
    Row::new("亥", "亥", "hai", "亥", "해", "Hợi"),
];

#[cfg(test)]
const SCOPE: [Row; 6] = [
    Row::new("大限", "大限", "decadal", "大限", "대한", "Đại Hạn"),
    Row::new("流年", "流年", "yearly", "流年", "유년", "Lưu Niên"),
    Row::new("流月", "流月", "monthly", "流月", "유월", "Lưu Nguyệt"),
    Row::new("流日", "流日", "daily", "流日", "유일", "Lưu Nhật"),
    Row::new("流时", "流時", "hourly", "流時", "유시", "Lưu Thì"),
    Row::new("小限", "小限", "age", "小限", "소한", "Tiểu Hạn"),
];

const ZODIAC: [Row; 12] = [
    Row::new("鼠", "鼠", "rat", "鼠", "쥐", "Chuột"),
    Row::new("牛", "牛", "ox", "牛", "소", "Trâu"),
    Row::new("虎", "虎", "tiger", "虎", "호랑이", "Hổ"),
    Row::new("兔", "兔", "rabbit", "兎", "토끼", "Mèo"),
    Row::new("龙", "龍", "dragon", "龍", "용", "Rồng"),
    Row::new("蛇", "蛇", "snake", "蛇", "뱀", "Rắn"),
    Row::new("马", "馬", "horse", "馬", "말", "Ngựa"),
    Row::new("羊", "羊", "sheep", "羊", "양", "Dê"),
    Row::new("猴", "猴", "monkey", "猿", "원숭이", "Khỉ"),
    Row::new("鸡", "雞", "rooster", "雞", "닭", "Gà"),
    Row::new("狗", "狗", "dog", "犬", "개", "Chó"),
    Row::new("猪", "豬", "pig", "豚", "돼지", "Lợn"),
];

const SIGN: [Row; 12] = [
    Row::new(
        "白羊座",
        "白羊座",
        "aries",
        "おひつじ座",
        "백양궁",
        "Cung Bạch Dương",
    ),
    Row::new(
        "金牛座",
        "金牛座",
        "taurus",
        "おうし座",
        "금우궁",
        "Cung Kim Ngưu",
    ),
    Row::new(
        "双子座",
        "雙子座",
        "gemini",
        "ふたご座",
        "쌍아궁",
        "Cung Song Tử",
    ),
    Row::new(
        "巨蟹座",
        "巨蟹座",
        "cancer",
        "かに座",
        "거해궁",
        "Cung Cự Giải",
    ),
    Row::new("狮子座", "獅子座", "leo", "しし座", "사자궁", "Cung Sư Tử"),
    Row::new(
        "处女座",
        "處女座",
        "virgo",
        "おとめ座",
        "처녀궁",
        "Cung Xử Nữ",
    ),
    Row::new(
        "天秤座",
        "天秤座",
        "libra",
        "てんびん座",
        "천칭궁",
        "Cung Thiên Bình",
    ),
    Row::new(
        "天蝎座",
        "天蠍座",
        "scorpio",
        "さそり座",
        "천갈궁",
        "Cung Thiên Yết",
    ),
    Row::new(
        "射手座",
        "射手座",
        "sagittarius",
        "いて座",
        "인마궁",
        "Cung Xạ Thủ",
    ),
    Row::new(
        "摩羯座",
        "摩羯座",
        "capricorn",
        "やぎ座",
        "마갈궁",
        "Cung Ma Kết",
    ),
    Row::new(
        "水瓶座",
        "水瓶座",
        "aquarius",
        "みずがめ座",
        "보병궁",
        "Cung Thủy Bình",
    ),
    Row::new(
        "双鱼座",
        "雙魚座",
        "pisces",
        "うお座",
        "쌍어궁",
        "Cung Song Ngư",
    ),
];

#[cfg(test)]
const COMMON: [Row; 44] = [
    Row::new("大限", "大限", "decadal", "大限", "대한", "Đại Hạn"),
    Row::new("童限", "童限", "childhood", "子供", "어린", "đứa trẻ Hạn"),
    Row::new("流年", "流年", "yearly", "流年", "유년", "Lưu Niên"),
    Row::new("流月", "流月", "monthly", "流月", "유월", "Lưu Nguyệt"),
    Row::new("流日", "流日", "daily", "流日", "유일", "Lưu Nhật"),
    Row::new("流时", "流時", "hourly", "流時", "유시", "Lưu Thì"),
    Row::new("小限", "小限", "age", "小限", "소한", "Tiểu Hạn"),
    Row::new("鼠", "鼠", "rat", "鼠", "쥐", "Chuột"),
    Row::new("牛", "牛", "ox", "牛", "소", "Trâu"),
    Row::new("虎", "虎", "tiger", "虎", "호랑이", "Hổ"),
    Row::new("兔", "兔", "rabbit", "兎", "토끼", "Mèo"),
    Row::new("龙", "龍", "dragon", "龍", "용", "Rồng"),
    Row::new("蛇", "蛇", "snake", "蛇", "뱀", "Rắn"),
    Row::new("马", "馬", "horse", "馬", "말", "Ngựa"),
    Row::new("羊", "羊", "sheep", "羊", "양", "Dê"),
    Row::new("猴", "猴", "monkey", "猿", "원숭이", "Khỉ"),
    Row::new("鸡", "雞", "rooster", "雞", "닭", "Gà"),
    Row::new("狗", "狗", "dog", "犬", "개", "Chó"),
    Row::new("猪", "豬", "pig", "豚", "돼지", "Lợn"),
    Row::new(
        "早子时",
        "早子時",
        "early Rat hour",
        "早子時",
        "아침 자시",
        "Giờ tý sớm",
    ),
    Row::new("丑时", "丑時", "Ox hour", "丑時", "축시", "Giờ sửu"),
    Row::new("寅时", "寅時", "Tiger hour", "寅時", "인시", "Giờ dần"),
    Row::new("卯时", "卯時", "Rabbit hour", "卯時", "묘시", "Giờ mão"),
    Row::new("辰时", "辰時", "Dragon hour", "辰時", "진시", "Giờ thìn"),
    Row::new("巳时", "巳時", "Snake hour", "巳時", "사시", "Giờ tỵ"),
    Row::new("午时", "午時", "Horse hour", "午時", "오시", "Giờ ngọ"),
    Row::new("未时", "未時", "Goat hour", "未時", "미시", "Giờ mùi"),
    Row::new("申时", "申時", "Monkey hour", "申時", "신시", "Giờ thân"),
    Row::new("酉时", "酉時", "Rooster hour", "酉時", "유시", "Giờ dậu"),
    Row::new("戌时", "戌時", "Dog hour", "戌時", "술시", "Giờ tuất"),
    Row::new("亥时", "亥時", "Pig hour", "亥時", "해시", "Giờ hợi"),
    Row::new(
        "晚子时",
        "晚子時",
        "late Rat hour",
        "晚子時",
        "밤에 자시",
        "Giờ tý muộn",
    ),
    Row::new(
        "白羊座",
        "白羊座",
        "aries",
        "おひつじ座",
        "백양궁",
        "Cung Bạch Dương",
    ),
    Row::new(
        "金牛座",
        "金牛座",
        "taurus",
        "おうし座",
        "금우궁",
        "Cung Kim Ngưu",
    ),
    Row::new(
        "双子座",
        "雙子座",
        "gemini",
        "ふたご座",
        "쌍아궁",
        "Cung Song Tử",
    ),
    Row::new(
        "巨蟹座",
        "巨蟹座",
        "cancer",
        "かに座",
        "거해궁",
        "Cung Cự Giải",
    ),
    Row::new("狮子座", "獅子座", "leo", "しし座", "사자궁", "Cung Sư Tử"),
    Row::new(
        "处女座",
        "處女座",
        "virgo",
        "おとめ座",
        "처녀궁",
        "Cung Xử Nữ",
    ),
    Row::new(
        "天秤座",
        "天秤座",
        "libra",
        "てんびん座",
        "천칭궁",
        "Cung Thiên Bình",
    ),
    Row::new(
        "天蝎座",
        "天蠍座",
        "scorpio",
        "さそり座",
        "천갈궁",
        "Cung Thiên Yết",
    ),
    Row::new(
        "射手座",
        "射手座",
        "sagittarius",
        "いて座",
        "인마궁",
        "Cung Xạ Thủ",
    ),
    Row::new(
        "摩羯座",
        "摩羯座",
        "capricorn",
        "やぎ座",
        "마갈궁",
        "Cung Ma Kết",
    ),
    Row::new(
        "水瓶座",
        "水瓶座",
        "aquarius",
        "みずがめ座",
        "보병궁",
        "Cung Thủy Bình",
    ),
    Row::new(
        "双鱼座",
        "雙魚座",
        "pisces",
        "うお座",
        "쌍어궁",
        "Cung Song Ngư",
    ),
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use iztro::{
        Brightness, EarthlyBranch, FiveElementBureau, Gender, HeavenlyStem, Mutagen, PalaceName,
        Scope, StarName, WesternZodiac, known_star_metadata_table,
    };

    use super::*;

    const LANGUAGES: [Language; 6] = [
        Language::ZhCn,
        Language::ZhTw,
        Language::EnUs,
        Language::JaJp,
        Language::KoKr,
        Language::ViVn,
    ];

    #[test]
    fn parses_only_the_six_contract_languages() {
        for language in LANGUAGES {
            assert_eq!(Language::parse(language.code()), Ok(language));
        }
        assert_eq!(Language::default(), Language::ZhCn);
        assert_eq!(
            Language::parse("en").unwrap_err(),
            "Unsupported language. Use zh-CN, zh-TW, en-US, ja-JP, ko-KR, or vi-VN."
        );
    }

    #[test]
    fn reports_upstream_namespace_coverage() {
        assert_eq!(entry_count(LocaleKind::Palace), 14);
        assert_eq!(entry_count(LocaleKind::Star), 162);
        assert_eq!(entry_count(LocaleKind::Decorative), 162);
        assert_eq!(entry_count(LocaleKind::Brightness), 7);
        assert_eq!(entry_count(LocaleKind::Mutagen), 4);
        assert_eq!(entry_count(LocaleKind::FiveElement), 5);
        assert_eq!(entry_count(LocaleKind::Gender), 2);
        assert_eq!(entry_count(LocaleKind::Stem), 10);
        assert_eq!(entry_count(LocaleKind::Branch), 12);
        assert_eq!(entry_count(LocaleKind::Scope), 6);
        assert_eq!(entry_count(LocaleKind::Zodiac), 12);
        assert_eq!(entry_count(LocaleKind::Sign), 12);
        assert_eq!(entry_count(LocaleKind::Common), 44);
    }

    #[test]
    fn every_row_has_a_unique_canonical_key_and_six_nonempty_values() {
        for table in [
            PALACE.as_slice(),
            STAR.as_slice(),
            BRIGHTNESS.as_slice(),
            MUTAGEN.as_slice(),
            FIVE_ELEMENT.as_slice(),
            GENDER.as_slice(),
            STEM.as_slice(),
            BRANCH.as_slice(),
            SCOPE.as_slice(),
            ZODIAC.as_slice(),
            SIGN.as_slice(),
            COMMON.as_slice(),
        ] {
            let mut canonical = HashSet::new();
            for row in table {
                assert!(canonical.insert(row.canonical()));
                assert!(row.values.iter().all(|value| !value.is_empty()));
            }
        }
    }

    #[test]
    fn preserves_representative_values_verbatim_across_all_languages() {
        assert_eq!(
            LANGUAGES.map(|language| language.palace(PalaceName::Career)),
            ["官禄", "官祿", "career", "官祿", "관록", "Quan Lộc"]
        );
        assert_eq!(
            LANGUAGES.map(|language| language.star(StarName::ZiWei)),
            ["紫微", "紫微", "emperor", "紫微", "자미", "Tử Vi"]
        );
        assert_eq!(
            LANGUAGES.map(|language| language.brightness(Brightness::Temple)),
            ["庙", "廟", "[+3]", "廟", "[+3]", "Miếu"]
        );
        assert_eq!(
            LANGUAGES.map(|language| language.mutagen(Mutagen::Ji)),
            ["忌", "忌", "D", "忌", "기", "Kỵ"]
        );
        assert_eq!(
            LANGUAGES.map(|language| language.five_element_bureau(FiveElementBureau::Fire6)),
            [
                "火六局",
                "火六局",
                "fire 6th",
                "火の六局",
                "화육국",
                "Hỏa Lục Cục",
            ]
        );
        assert_eq!(
            LANGUAGES.map(|language| language.gender(Gender::Female)),
            ["女", "女", "female", "女", "여자", "Nữ"]
        );
        assert_eq!(
            LANGUAGES.map(|language| language.heavenly_stem(HeavenlyStem::Jia)),
            ["甲", "甲", "jia", "甲", "갑", "Giáp"]
        );
        assert_eq!(
            LANGUAGES.map(|language| language.earthly_branch(EarthlyBranch::Chen)),
            ["辰", "辰", "chen", "辰", "진", "Thìn"]
        );
        assert_eq!(
            LANGUAGES.map(|language| language.scope(Scope::Decadal)),
            ["大限", "大限", "decadal", "大限", "대한", "Đại Hạn"]
        );
        assert_eq!(
            LANGUAGES.map(|language| language.zodiac_animal(EarthlyBranch::Chen)),
            ["龙", "龍", "dragon", "龍", "용", "Rồng"]
        );
        assert_eq!(
            LANGUAGES.map(|language| language.western_zodiac(WesternZodiac::Scorpio)),
            [
                "天蝎座",
                "天蠍座",
                "scorpio",
                "さそり座",
                "천갈궁",
                "Cung Thiên Yết",
            ]
        );
    }

    #[test]
    fn all_rust_known_stars_resolve_through_the_upstream_star_table() {
        for metadata in known_star_metadata_table() {
            let canonical = zh_cn::star_name_zh(metadata.name());
            assert_eq!(metadata.chinese_name(), canonical);
            assert!(
                Language::EnUs.lookup(LocaleKind::Star, canonical).is_some(),
                "missing locale row for {} ({})",
                metadata.key(),
                canonical
            );
        }
    }

    #[test]
    fn explicit_upstream_gaps_fall_back_without_inventing_translations() {
        assert_eq!(Language::EnUs.scope(Scope::Natal), "本命");
        assert_eq!(Language::EnUs.common("晚子时"), "late Rat hour");
        assert_eq!(
            Language::ViVn.translate(LocaleKind::Palace, "未知宫"),
            "未知宫"
        );
        assert_eq!(Language::EnUs.brightness(Brightness::Unknown), "");
    }
}

use unicode_normalization::UnicodeNormalization;

use crate::dto::{SpecificationRow, SpecificationSection};
use crate::error::AppError;

pub const SCHEMA_VERSION: u8 = 1;
pub const MAX_COMPARISON_DEVICES: usize = 3;

pub const SPEC_KEYS: [&str; 28] = [
    "weight",
    "storage",
    "stylus",
    "displayPanel",
    "displaySize",
    "displayResolution",
    "refreshRate",
    "displayPeakBrightness",
    "displayLamination",
    "displayAntiReflective",
    "displayColorGamut",
    "displayContrastRatio",
    "displaySupplier",
    "displayFeatures",
    "processor",
    "memory",
    "wiredConnection",
    "speakers",
    "rearCameras",
    "telephoto",
    "digitalZoom",
    "frontCamera",
    "videoRecording",
    "videoPlayback",
    "wireless",
    "biometrics",
    "waterResistance",
    "sim",
];

/// Foldables can carry up to two extra displays besides the main one. Each
/// sub display is all-or-none (every key of its group) and sub display 2
/// requires sub display 1.
pub const SUB_DISPLAY_KEY_GROUPS: [[&str; 11]; 2] = [
    [
        "sub1DisplayPanel",
        "sub1DisplaySize",
        "sub1DisplayResolution",
        "sub1RefreshRate",
        "sub1PeakBrightness",
        "sub1DisplayLamination",
        "sub1DisplayAntiReflective",
        "sub1DisplayColorGamut",
        "sub1DisplayContrastRatio",
        "sub1DisplaySupplier",
        "sub1DisplayFeatures",
    ],
    [
        "sub2DisplayPanel",
        "sub2DisplaySize",
        "sub2DisplayResolution",
        "sub2RefreshRate",
        "sub2PeakBrightness",
        "sub2DisplayLamination",
        "sub2DisplayAntiReflective",
        "sub2DisplayColorGamut",
        "sub2DisplayContrastRatio",
        "sub2DisplaySupplier",
        "sub2DisplayFeatures",
    ],
];

/// Whether `provided` (a device's spec keys) is the required set plus a valid
/// combination of sub display groups.
/// Optional display name per sub display (e.g. "커버 디스플레이"); only valid
/// together with its group.
pub const SUB_DISPLAY_NAME_KEYS: [&str; 2] = ["sub1DisplayName", "sub2DisplayName"];

pub fn spec_keys_are_valid(category: &str, provided: &std::collections::HashSet<&str>) -> bool {
    let required = spec_keys_for_category(category);
    if !required.iter().all(|key| provided.contains(key)) {
        return false;
    }
    let mut previous_present = true;
    let mut extra = 0;
    for (group, name_key) in SUB_DISPLAY_KEY_GROUPS.iter().zip(SUB_DISPLAY_NAME_KEYS) {
        let present = group.iter().filter(|key| provided.contains(*key)).count();
        if present != 0 && present != group.len() {
            return false;
        }
        if present != 0 && !previous_present {
            return false;
        }
        let name_present = usize::from(provided.contains(name_key));
        if name_present != 0 && present == 0 {
            return false;
        }
        previous_present = present != 0;
        extra += present + name_present;
    }
    provided.len() == required.len() + extra
}

pub fn spec_keys_for_category(_category: &str) -> &'static [&'static str] {
    &SPEC_KEYS
}

pub fn normalize_route_identifier(value: &str) -> Result<String, AppError> {
    if value.chars().count() > 160 {
        return Err(AppError::Validation(
            "identifier must be at most 160 characters".into(),
        ));
    }

    let normalized: String = value.nfkc().flat_map(char::to_lowercase).collect();
    let mut result = String::new();
    let mut pending_separator = false;

    for character in normalized.trim().chars() {
        if character.is_whitespace() || character == '_' {
            pending_separator = !result.is_empty();
        } else {
            if pending_separator {
                result.push('-');
                pending_separator = false;
            }
            result.push(character);
        }
    }

    if result.is_empty() {
        return Err(AppError::Validation("identifier must not be empty".into()));
    }
    if result.chars().count() > 160 {
        return Err(AppError::Validation(
            "normalized identifier must be at most 160 characters".into(),
        ));
    }

    Ok(result)
}

// Korean product-line names for devices whose stored name/aliases are only
// ever in English (e.g. "iPhone 16", "Galaxy S24") — without this, a Korean
// query for either never matches anything, since nothing else in the data
// carries the Korean spelling. Expanding to the English form here means any
// current or future device under that product line is searchable by either
// spelling, with no per-device alias data to add or keep up to date.
//
// Replacement runs in order, so the bare syllables "폴"/"플" (what a user has
// typed so far on the way to 폴드/플립) must stay after their full words.
const SEARCH_TERM_SYNONYMS: [(&str, &str); 8] = [
    ("아이폰", "iphone"),
    ("갤럭시", "galaxy"),
    ("폴드", "fold"),
    ("플립", "flip"),
    ("폴", "fold"),
    ("플", "flip"),
    ("캐논", "canon"),
    ("삼성", "samsung"),
];

pub fn expand_search_synonyms(value: &str) -> String {
    let mut result = value.to_string();
    for (korean, english) in SEARCH_TERM_SYNONYMS {
        if result.contains(korean) {
            result = result.replace(korean, english);
        }
    }
    result
}

// Hangul initial-consonant (초성) shortcuts for foldables/flips. A single
// substring can't cover both "fold" and "flip", so a query made only of these
// jamo expands to several alternative keys instead of one.
const SEARCH_INITIAL_ALTERNATIVES: [(&str, &[&str]); 3] = [
    ("ㅍ", &["fold", "flip"]),
    ("ㅍㄷ", &["fold"]),
    ("ㅍㄹ", &["flip"]),
];

// Alternative search keys for a raw query; a device matches if any of them is
// a substring of one of its identifiers. Empty means "no search filter".
pub fn search_keys(value: &str) -> Vec<String> {
    let key = normalize_search_term(value);
    if key.is_empty() {
        return Vec::new();
    }
    for (initials, alternatives) in SEARCH_INITIAL_ALTERNATIVES {
        if key == normalize_search_term(initials) {
            return alternatives
                .iter()
                .map(|alternative| (*alternative).to_string())
                .collect();
        }
    }
    vec![key]
}

pub fn normalize_search_term(value: &str) -> String {
    let normalized: String = value
        .nfkc()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_alphanumeric() || matches!(character, ',' | '+'))
        .collect();
    expand_search_synonyms(&normalized)
}

pub fn specification_sections() -> Vec<SpecificationSection> {
    vec![
        section(
            "basic",
            "기본 정보",
            "핵심 하드웨어와 기본 사양",
            &[
                ("processor", "프로세서 (AP)"),
                ("memory", "메모리"),
                ("rearCameras", "카메라"),
                ("displaySize", "디스플레이 크기"),
                ("dimensions", "크기"),
                ("weight", "무게"),
                ("wiredConnection", "단자"),
                ("biometrics", "생체인식"),
                ("waterResistance", "방수방진"),
                ("speakers", "스피커"),
                ("operatingSystem", "운영체제"),
                ("releaseDate", "출시일"),
            ],
        ),
        section(
            "display",
            "디스플레이",
            "화면 크기와 표현 방식",
            &[
                ("displayPanel", "패널"),
                ("displaySize", "화면 크기"),
                ("displayResolution", "해상도"),
                ("refreshRate", "재생률"),
                ("displayFeatures", "주요 기능"),
            ],
        ),
        section(
            "performance",
            "성능",
            "칩, 메모리, 연결 단자",
            &[
                ("processor", "프로세서"),
                ("memory", "메모리"),
                ("storage", "저장 용량"),
                ("wiredConnection", "유선 연결"),
            ],
        ),
        section(
            "camera",
            "카메라",
            "후면 시스템, 줌, 동영상",
            &[
                ("rearCameras", "후면 카메라"),
                ("telephoto", "망원"),
                ("digitalZoom", "디지털 줌"),
                ("frontCamera", "전면 카메라"),
                ("videoRecording", "동영상 촬영"),
            ],
        ),
        section(
            "battery",
            "배터리와 충전",
            "제조사 시험 기준",
            &[
                ("batteryCapacity", "배터리 용량"),
                ("videoPlayback", "동영상 재생"),
                ("fastCharging", "급속 충전"),
                ("wirelessCharging", "무선 충전"),
            ],
        ),
        section(
            "connectivity",
            "연결과 내구성",
            "네트워크, 생체 인증, 방수",
            &[
                ("wireless", "무선 연결"),
                ("biometrics", "생체 인증"),
                ("waterResistance", "방수·방진"),
            ],
        ),
    ]
}

fn section(
    key: &'static str,
    title: &'static str,
    description: &'static str,
    rows: &[(&'static str, &'static str)],
) -> SpecificationSection {
    SpecificationSection {
        key,
        title,
        description,
        rows: rows
            .iter()
            .map(|(key, label)| SpecificationRow { key, label })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_normalization_matches_frontend_contract() {
        assert_eq!(
            normalize_route_identifier(" Galaxy_S24 ").unwrap(),
            "galaxy-s24"
        );
        assert_eq!(
            normalize_route_identifier("iPhone19,3").unwrap(),
            "iphone19,3"
        );
    }

    #[test]
    fn search_normalization_preserves_plus_and_comma() {
        assert_eq!(normalize_search_term("Galaxy S24+"), "galaxys24+");
        assert_eq!(normalize_search_term("SM-S921"), "sms921");
        assert_eq!(normalize_search_term("iPhone19,3"), "iphone19,3");
    }

    #[test]
    fn search_normalization_expands_korean_product_line_names() {
        assert_eq!(normalize_search_term("아이폰"), "iphone");
        assert_eq!(normalize_search_term("아이폰 16"), "iphone16");
        assert_eq!(normalize_search_term("갤럭시 S24"), "galaxys24");
        assert_eq!(normalize_search_term("폴드"), "fold");
        assert_eq!(normalize_search_term("갤럭시 Z 폴드7"), "galaxyzfold7");
        assert_eq!(normalize_search_term("플립"), "flip");
        assert_eq!(normalize_search_term("갤럭시 Z 플립7"), "galaxyzflip7");
        assert_eq!(normalize_search_term("폴"), "fold");
        assert_eq!(normalize_search_term("플"), "flip");
        assert_eq!(expand_search_synonyms("아이폰 16 찾기"), "iphone 16 찾기");
    }

    #[test]
    fn search_keys_expand_fold_flip_initials() {
        assert_eq!(search_keys("ㅍ"), vec!["fold", "flip"]);
        assert_eq!(search_keys(" ㅍㄷ "), vec!["fold"]);
        assert_eq!(search_keys("ㅍㄹ"), vec!["flip"]);
        assert_eq!(search_keys("폴드"), vec!["fold"]);
        assert_eq!(search_keys("ㅍ7"), vec![normalize_search_term("ㅍ7")]);
        assert!(search_keys("  ").is_empty());
    }

    #[test]
    fn schema_contains_expected_rows() {
        let sections = specification_sections();
        assert_eq!(sections.len(), 6);
        assert_eq!(
            sections
                .iter()
                .map(|section| section.rows.len())
                .sum::<usize>(),
            33
        );
    }
}

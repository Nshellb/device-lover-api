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

// Words a user is likely to be partway through typing. A whole query that is a
// leading part of one of these — as full jamo ("아", "아이ㅍ") or as initial
// consonants only ("ㅇ", "ㅇㅇㅍ") — expands to the English key, so results
// show up from the first keystroke instead of only once the word is complete.
// One query can lead several words ("ㅍ" -> fold and flip), hence several keys.
const SEARCH_TYPING_TARGETS: [(&str, &str); 4] = [
    ("아이폰", "iphone"),
    ("갤럭시", "galaxy"),
    ("폴드", "fold"),
    ("플립", "flip"),
];

// Whether `typed` (NFD jamo) leads `word`, either as its full jamo sequence or
// as its initial consonants (the leading-consonant block U+1100..=U+115F).
fn is_leading_part_of(typed: &[char], word: &str) -> bool {
    let jamo: Vec<char> = word.nfd().collect();
    let initials: Vec<char> = jamo
        .iter()
        .copied()
        .filter(|jamo| ('\u{1100}'..='\u{115F}').contains(jamo))
        .collect();
    jamo.starts_with(typed) || initials.starts_with(typed)
}

// Alternative search keys for a raw query; a device matches if any of them is
// a substring of one of its identifiers. Empty means "no search filter".
pub fn search_keys(value: &str) -> Vec<String> {
    let typed = fold_search_text(value);
    if typed.is_empty() {
        return Vec::new();
    }
    let typed_jamo: Vec<char> = typed.nfd().collect();
    let keys: Vec<String> = SEARCH_TYPING_TARGETS
        .iter()
        .filter(|(word, _)| is_leading_part_of(&typed_jamo, word))
        .map(|(_, english)| (*english).to_string())
        .collect();
    if keys.is_empty() {
        vec![expand_search_synonyms(&typed)]
    } else {
        keys
    }
}

// NFKC + lowercase, keeping only what search keys are made of.
fn fold_search_text(value: &str) -> String {
    value
        .nfkc()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_alphanumeric() || matches!(character, ',' | '+'))
        .collect()
}

pub fn normalize_search_term(value: &str) -> String {
    expand_search_synonyms(&fold_search_text(value))
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
    fn search_keys_expand_partially_typed_fold_flip() {
        // Both words share the leading ㅍ.
        assert_eq!(search_keys("ㅍ"), vec!["fold", "flip"]);
        // Every state on the way to typing each word, as initials or as syllables.
        for typed in ["ㅍㄷ", " ㅍㄷ ", "포", "폴", "폴ㄷ", "폴드"] {
            assert_eq!(search_keys(typed), vec!["fold"], "typed {typed}");
        }
        for typed in ["ㅍㄹ", "프", "플", "플ㄹ", "플리", "플립"] {
            assert_eq!(search_keys(typed), vec!["flip"], "typed {typed}");
        }
    }

    #[test]
    fn search_keys_expand_partially_typed_iphone() {
        for typed in ["ㅇ", "아", "ㅇㅇ", "아이", "ㅇㅇㅍ", "아이ㅍ", "아이포", "아이폰"] {
            assert_eq!(search_keys(typed), vec!["iphone"], "typed {typed}");
        }
        assert_eq!(search_keys("아이폰 16"), vec!["iphone16"]);
        // Leading syllables shared with other words must not be taken for iPhone.
        assert_eq!(search_keys("아이패드"), vec!["아이패드"]);
    }

    #[test]
    fn search_keys_expand_partially_typed_galaxy() {
        for typed in ["ㄱ", "갤", "ㄱㄹ", "갤ㄹ", "갤러", "ㄱㄹㅅ", "갤럭", "갤럭시"] {
            assert_eq!(search_keys(typed), vec!["galaxy"], "typed {typed}");
        }
        assert_eq!(search_keys("갤럭시 Z 폴드7"), vec!["galaxyzfold7"]);
        // A syllable that only shares the initial consonant is not a Galaxy.
        assert_eq!(search_keys("가"), vec!["가"]);
    }

    #[test]
    fn search_keys_pass_other_queries_through() {
        assert_eq!(search_keys("ㅍ7"), vec![normalize_search_term("ㅍ7")]);
        assert_eq!(search_keys("갤럭시 S24"), vec!["galaxys24"]);
        assert_eq!(search_keys("iPhone"), vec!["iphone"]);
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

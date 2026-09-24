use unicode_normalization::UnicodeNormalization;

use crate::dto::{SpecificationRow, SpecificationSection};
use crate::error::AppError;

pub const SCHEMA_VERSION: u8 = 1;
pub const MAX_COMPARISON_DEVICES: usize = 3;

pub const SPEC_KEYS: [&str; 27] = [
    "operatingSystem",
    "colors",
    "dimensions",
    "weight",
    "storage",
    "stylus",
    "displayPanel",
    "displaySize",
    "displayResolution",
    "refreshRate",
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
    "batteryCapacity",
    "videoPlayback",
    "fastCharging",
    "wirelessCharging",
    "wireless",
    "biometrics",
    "waterResistance",
];

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

pub fn normalize_search_term(value: &str) -> String {
    value
        .nfkc()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_alphanumeric() || matches!(character, ',' | '+'))
        .collect()
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
                ("displaySize", "디스플레이 크기"),
                ("dimensions", "크기"),
                ("weight", "무게"),
                ("wiredConnection", "단자"),
                ("biometrics", "생체인식"),
                ("waterResistance", "방수방진"),
                ("speakers", "스피커"),
                ("operatingSystem", "운영체제"),
                ("colors", "색상"),
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

//! 언어 감지.
//!
//! ATS의 사용 시나리오에서 원문은 거의 항상 한국어다. 그래서 무거운 감지 모델 대신
//! "한글 글자 비율"로 번역 여부만 판단하고, 실제 원문 언어 판별은 공급자(DeepL)의
//! 자동 감지에 맡긴다.

/// 공백, 숫자, 구두점, 이모지 등을 뺀 "문자" 중 한글이 차지하는 비율 (0.0 ~ 1.0).
pub fn hangul_ratio(text: &str) -> f32 {
    let mut letters = 0u32;
    let mut hangul = 0u32;
    for c in text.chars().filter(|c| c.is_alphabetic()) {
        letters += 1;
        if is_hangul(c) {
            hangul += 1;
        }
    }
    if letters == 0 {
        0.0
    } else {
        hangul as f32 / letters as f32
    }
}

fn is_hangul(c: char) -> bool {
    matches!(c,
        '\u{AC00}'..='\u{D7A3}' // 완성형 음절
        | '\u{1100}'..='\u{11FF}' // 자모
        | '\u{3130}'..='\u{318F}' // 호환용 자모 (ㄱ, ㅏ ...)
    )
}

/// 한국어로 볼 최소 한글 비율. "ㅋㅋ lol" 같은 혼용 문장도 번역 대상에 넣기 위해 낮게 잡았다.
pub const KOREAN_THRESHOLD: f32 = 0.3;

/// 이 텍스트를 번역해야 하는지 판단한다.
///
/// `only_korean`이 true면 한국어로 보이는 텍스트만 번역한다. 이미 번역된 문장에
/// 단축키를 한 번 더 눌러도 이중 번역되지 않게 하는 안전장치다.
pub fn should_translate(text: &str, only_korean: bool) -> bool {
    if text.trim().is_empty() {
        return false;
    }
    !only_korean || hangul_ratio(text) >= KOREAN_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_korean_is_one() {
        assert_eq!(hangul_ratio("안녕하세요"), 1.0);
    }

    #[test]
    fn ignores_punctuation_digits_and_emoji() {
        assert_eq!(hangul_ratio("안녕!! 123 😀"), 1.0);
    }

    #[test]
    fn english_is_zero() {
        assert_eq!(hangul_ratio("hello world"), 0.0);
    }

    #[test]
    fn empty_is_zero() {
        assert_eq!(hangul_ratio("   "), 0.0);
    }

    #[test]
    fn jamo_counts_as_hangul() {
        assert_eq!(hangul_ratio("ㅋㅋㅋ"), 1.0);
    }

    #[test]
    fn mixed_sentence_crosses_threshold() {
        assert!(should_translate("이 기능 진짜 good idea 인듯", true));
    }

    #[test]
    fn already_translated_text_is_skipped() {
        assert!(!should_translate("This feature is a really good idea", true));
    }

    #[test]
    fn non_korean_is_translated_when_filter_off() {
        assert!(should_translate("This is English", false));
    }

    #[test]
    fn blank_is_never_translated() {
        assert!(!should_translate("  \n ", false));
    }
}

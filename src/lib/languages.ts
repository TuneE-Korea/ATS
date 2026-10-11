// DeepL target_lang 코드 기준. Azure는 Rust 쪽에서 자기 형식으로 변환한다.
// 전체 목록: https://developers.deepl.com/docs/getting-started/supported-languages
export const TARGET_LANGUAGES = [
  { code: "EN-US", label: "영어 (미국)" },
  { code: "EN-GB", label: "영어 (영국)" },
  { code: "JA", label: "일본어" },
  { code: "ZH-HANS", label: "중국어 (간체)" },
  { code: "ZH-HANT", label: "중국어 (번체)" },
  { code: "ES", label: "스페인어" },
  { code: "FR", label: "프랑스어" },
  { code: "DE", label: "독일어" },
  { code: "PT-BR", label: "포르투갈어 (브라질)" },
  { code: "RU", label: "러시아어" },
] as const;

export function languageLabel(code: string): string {
  return TARGET_LANGUAGES.find((l) => l.code === code)?.label ?? code;
}

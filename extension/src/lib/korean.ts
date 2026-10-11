// crates/ats-core/src/detect.rs와 같은 규칙. 한국어가 아닌 글에 서버 요청을 보내지 않으려고 확장에서도 먼저 거른다.

const HANGUL = /[가-힣ᄀ-ᇿ㄰-㆏]/u;
const LETTER = /\p{L}/u;

export const KOREAN_THRESHOLD = 0.3;
/** 이보다 짧으면 미리보기를 띄우지 않는다 ("ㅋㅋ" 같은 입력마다 요청하지 않기 위해). */
export const MIN_LENGTH = 2;

export function hangulRatio(text: string): number {
  let letters = 0;
  let hangul = 0;
  for (const ch of text) {
    if (!LETTER.test(ch)) continue;
    letters++;
    if (HANGUL.test(ch)) hangul++;
  }
  return letters === 0 ? 0 : hangul / letters;
}

export function shouldPreview(text: string): boolean {
  const t = text.trim();
  return [...t].length >= MIN_LENGTH && hangulRatio(t) >= KOREAN_THRESHOLD;
}

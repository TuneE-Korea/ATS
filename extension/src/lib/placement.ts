export interface Rect {
  top: number;
  bottom: number;
  left: number;
  width: number;
}

export interface Placement {
  top: number;
  left: number;
  width: number;
}

const GAP = 6;
const MIN_WIDTH = 240;
const MAX_WIDTH = 520;

/**
 * 미리보기 위치. 입력창 아래가 기본이고, 화면 아래에 공간이 없으면(채팅 앱 입력창은 보통 맨 아래) 위로 올린다.
 * 좌우는 화면 밖으로 나가지 않게 자른다.
 */
export function placePreview(field: Rect, previewHeight: number, viewport: { width: number; height: number }): Placement {
  const width = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, field.width), viewport.width - 2 * GAP);
  const left = Math.min(Math.max(GAP, field.left), viewport.width - width - GAP);
  const below = field.bottom + GAP;
  const fitsBelow = below + previewHeight <= viewport.height - GAP;
  const top = fitsBelow ? below : Math.max(GAP, field.top - GAP - previewHeight);
  return { top, left, width };
}

// 입력창을 지켜보다가 한국어 입력이 멈추면 번역 미리보기를 띄운다. Tab으로 적용, Esc로 닫기.
//
// 입력창 내용을 직접 바꾸는 건 사용자가 Tab을 눌렀을 때 한 번뿐이다.
// 타이핑 중에 바꾸면 한글 조합(IME)과 커서 위치가 깨지기 때문이다.

import { shouldPreview } from "./lib/korean";
import { PROVIDER_LABEL, type Request, type ServerStatus, type StatusResponse, type TranslateResponse } from "./lib/messages";
import { placePreview } from "./lib/placement";

type Editable = HTMLInputElement | HTMLTextAreaElement | HTMLElement;

const STATUS_TTL_MS = 10_000;

let serverStatus: { value: ServerStatus | null; at: number } = { value: null, at: 0 };
let field: Editable | null = null;
let timer: number | undefined;
let seq = 0;
/** 현재 미리보기가 어떤 원문에 대한 번역인지. 입력창 내용이 바뀌면 적용하지 않는다. */
let shown: { source: string; translated: string } | null = null;

// ---------- 입력창 다루기 ----------

function isEditable(el: EventTarget | null): el is Editable {
  if (el instanceof HTMLTextAreaElement) return !el.readOnly && !el.disabled;
  if (el instanceof HTMLInputElement) return ["text", "search", ""].includes(el.type) && !el.readOnly && !el.disabled;
  return el instanceof HTMLElement && el.isContentEditable;
}

function readText(el: Editable): string {
  return el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement ? el.value : el.innerText;
}

/** React 같은 프레임워크가 변경을 알아채도록, 프로토타입의 value setter를 직접 호출하고 input 이벤트를 보낸다. */
function writeText(el: Editable, text: string) {
  if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) {
    const proto = el instanceof HTMLInputElement ? HTMLInputElement.prototype : HTMLTextAreaElement.prototype;
    Object.getOwnPropertyDescriptor(proto, "value")?.set?.call(el, text);
    el.dispatchEvent(new Event("input", { bubbles: true }));
    return;
  }
  // contenteditable(Discord, Slack, Reddit 등의 리치 에디터)은 자체 상태를 가지고 있어서 DOM을 직접 바꾸면 무시된다.
  // 전체 선택 후 insertText를 보내면 에디터가 사용자 입력처럼 처리한다.
  el.focus();
  const range = document.createRange();
  range.selectNodeContents(el);
  const sel = window.getSelection();
  sel?.removeAllRanges();
  sel?.addRange(range);
  document.execCommand("insertText", false, text);
}

// ---------- 서비스 워커와 통신 ----------

function send<T>(msg: Request): Promise<T> {
  return chrome.runtime.sendMessage(msg) as Promise<T>;
}

async function getStatus(): Promise<ServerStatus | null> {
  if (Date.now() - serverStatus.at < STATUS_TTL_MS) return serverStatus.value;
  const res = await send<StatusResponse>({ type: "status" }).catch(() => null);
  serverStatus = { value: res?.ok ? res.status : null, at: Date.now() };
  return serverStatus.value;
}

// ---------- 미리보기 UI (Shadow DOM으로 페이지 CSS와 격리) ----------

const host = document.createElement("ats-preview");
const shadow = host.attachShadow({ mode: "closed" });
shadow.innerHTML = `
<style>
  :host { all: initial; }
  .box { position: fixed; z-index: 2147483647; box-sizing: border-box; padding: 10px 12px; border-radius: 10px;
    font: 14px/1.45 system-ui, "Malgun Gothic", sans-serif; color: #18181b; background: #fff;
    border: 1px solid #e4e4e7; box-shadow: 0 8px 24px rgba(0,0,0,.14); display: none; }
  .text { white-space: pre-wrap; word-break: break-word; }
  .meta { margin-top: 6px; font-size: 11px; color: #71717a; display: flex; gap: 8px; }
  .meta b { color: #4f46e5; font-weight: 600; }
  .muted { color: #71717a; }
  .error { color: #dc2626; font-size: 12px; }
  @media (prefers-color-scheme: dark) {
    .box { color: #f4f4f5; background: #18181b; border-color: #3f3f46; }
    .meta b { color: #a5b4fc; }
  }
</style>
<div class="box" role="status"><div class="text"></div><div class="meta"></div></div>`;
const box = shadow.querySelector<HTMLDivElement>(".box")!;
const textEl = shadow.querySelector<HTMLDivElement>(".text")!;
const metaEl = shadow.querySelector<HTMLDivElement>(".meta")!;

function render(kind: "loading" | "result" | "error", text: string, meta = "") {
  if (!field) return;
  if (!host.isConnected) document.documentElement.append(host);
  textEl.className = kind === "result" ? "text" : kind === "error" ? "text error" : "text muted";
  textEl.textContent = text;
  metaEl.innerHTML = meta;
  box.style.display = "block";
  reposition();
}

function reposition() {
  if (!field || box.style.display === "none") return;
  const r = field.getBoundingClientRect();
  const p = placePreview(r, box.offsetHeight, { width: innerWidth, height: innerHeight });
  Object.assign(box.style, { top: `${p.top}px`, left: `${p.left}px`, width: `${p.width}px` });
}

function hide() {
  box.style.display = "none";
  shown = null;
}

// ---------- 흐름 ----------

async function preview(el: Editable) {
  const source = readText(el);
  if (!shouldPreview(source)) return hide();
  if (shown?.source === source) return;

  const id = ++seq;
  render("loading", "번역 중…");
  const res = await send<TranslateResponse>({
    type: "translate",
    text: source,
    domain: location.hostname,
    title: document.title,
  }).catch(() => null);

  // 응답을 기다리는 사이 더 입력했거나 다른 입력창으로 옮겼으면 버린다.
  if (id !== seq || el !== field || readText(el) !== source) return;
  if (!res) return render("error", "ATS 확장과 연결이 끊겼습니다. 페이지를 새로고침해 주세요.");
  if (!res.ok) return render("error", res.message);
  if (!res.result) return hide();

  shown = { source, translated: res.result.text };
  const where = res.result.profile ? ` · ${res.result.profile}` : "";
  render(
    "result",
    res.result.text,
    `<span><b>Tab</b> 적용 · <b>Esc</b> 닫기</span><span style="margin-left:auto">${PROVIDER_LABEL[res.result.provider]}${where}</span>`,
  );
}

async function schedule(el: Editable) {
  const s = await getStatus();
  if (s?.mode !== "realtime") return hide();
  clearTimeout(timer);
  timer = window.setTimeout(() => void preview(el), s.debounceMs);
}

document.addEventListener(
  "input",
  (e) => {
    // 조합 중인 글자도 입력창 값에 포함되므로 그대로 예약한다. 적용(Tab)만 조합이 끝난 뒤에 한다.
    if (!isEditable(e.target)) return;
    field = e.target;
    if (shown && readText(field) !== shown.source) shown = null;
    void schedule(field);
  },
  true,
);

document.addEventListener(
  "keydown",
  (e) => {
    if (!field || e.target !== field || box.style.display === "none") return;
    if (e.key === "Escape") {
      hide();
      return;
    }
    if (e.key !== "Tab" || e.shiftKey || e.isComposing || !shown) return;
    if (readText(field) !== shown.source) return;
    e.preventDefault();
    e.stopPropagation();
    const text = shown.translated;
    hide();
    clearTimeout(timer);
    writeText(field, text);
  },
  true,
);

document.addEventListener(
  "focusout",
  (e) => {
    if (e.target === field) {
      clearTimeout(timer);
      hide();
    }
  },
  true,
);

addEventListener("scroll", reposition, true);
addEventListener("resize", reposition);

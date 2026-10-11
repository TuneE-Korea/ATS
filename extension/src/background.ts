// 서비스 워커: content script의 요청을 받아 ATS 앱의 로컬 서버(127.0.0.1)로 전달한다.
// content script는 페이지 출처로 요청하게 되어 서버에서 거부되므로, 요청은 반드시 여기서 보낸다.

import type { Failure, Request, ServerStatus, StatusResponse, TranslateResponse, TranslateResult } from "./lib/messages";

const BASE = "http://127.0.0.1:47120";

async function getToken(): Promise<string> {
  const { token } = await chrome.storage.local.get("token");
  return typeof token === "string" ? token : "";
}

function failure(reason: Failure["reason"], message: string): Failure {
  return { ok: false, reason, message };
}

async function call(path: string, token: string, init?: RequestInit): Promise<Response | Failure> {
  try {
    const res = await fetch(BASE + path, {
      ...init,
      headers: { "content-type": "application/json", "x-ats-token": token, ...init?.headers },
    });
    if (res.status === 401 || res.status === 403) {
      return failure("unauthorized", "연결 토큰이 맞지 않습니다. 확장 옵션에서 토큰을 다시 넣어 주세요.");
    }
    return res;
  } catch {
    return failure("offline", "ATS 앱이 실행 중이 아닙니다.");
  }
}

async function status(token: string): Promise<StatusResponse> {
  const res = await call("/v1/status", token);
  if (!(res instanceof Response)) return res;
  return { ok: true, status: (await res.json()) as ServerStatus };
}

async function translate(text: string, domain: string, title: string): Promise<TranslateResponse> {
  const res = await call("/v1/translate", await getToken(), {
    method: "POST",
    body: JSON.stringify({ text, domain, title }),
  });
  if (!(res instanceof Response)) return res;
  if (res.status === 204) return { ok: true, result: null };
  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as { error?: string };
    return failure("failed", body.error ?? `번역 실패 (${res.status})`);
  }
  return { ok: true, result: (await res.json()) as TranslateResult };
}

chrome.runtime.onMessage.addListener((msg: Request, _sender, sendResponse) => {
  const work =
    msg.type === "status"
      ? getToken().then(status)
      : msg.type === "check"
        ? status(msg.token)
        : translate(msg.text, msg.domain, msg.title);
  void work.then(sendResponse);
  return true; // 비동기로 응답한다는 표시
});

chrome.action.onClicked.addListener(() => void chrome.runtime.openOptionsPage());

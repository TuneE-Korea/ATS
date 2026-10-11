import type { StatusResponse } from "./lib/messages";

const input = document.querySelector<HTMLInputElement>("#token")!;
const result = document.querySelector<HTMLParagraphElement>("#result")!;

void chrome.storage.local.get("token").then(({ token }) => {
  if (typeof token === "string") input.value = token;
});

document.querySelector("#save")!.addEventListener("click", async () => {
  const token = input.value.trim();
  await chrome.storage.local.set({ token });
  const res = (await chrome.runtime.sendMessage({ type: "check", token })) as StatusResponse;
  if (res.ok) {
    const mode = res.status.mode === "realtime" ? "실시간 미리보기 켜짐" : "단축키 모드 (앱에서 실시간 미리보기를 켜 주세요)";
    result.className = "ok";
    result.textContent = `연결됨: ATS ${res.status.version}, ${mode}`;
  } else {
    result.className = "err";
    result.textContent = res.message;
  }
});

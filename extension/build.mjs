// 확장 빌드: TS를 번들하고 manifest·옵션 페이지·아이콘을 dist로 복사한다.
// 사용: npm run build:ext → extension/dist 를 chrome://extensions 에서 "압축해제된 확장 프로그램"으로 로드
import { build } from "esbuild";
import { cpSync, mkdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const out = join(root, "dist");
rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

// 서비스 워커만 모듈(manifest의 "type": "module")이고, content script와 옵션 페이지는 일반 스크립트로 실행된다.
// 일반 스크립트를 esm으로 내보내면 최상위 변수가 페이지 전역(window.status 등)과 섞이므로 iife로 감싼다.
const common = { outdir: out, bundle: true, target: "chrome120", logLevel: "info" };
await build({ ...common, entryPoints: [join(root, "src", "background.ts")], format: "esm" });
await build({ ...common, entryPoints: ["content", "options"].map((n) => join(root, "src", `${n}.ts`)), format: "iife" });

for (const f of ["manifest.json", "options.html", "icons"]) cpSync(join(root, f), join(out, f), { recursive: true });

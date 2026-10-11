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

await build({
  entryPoints: ["background", "content", "options"].map((n) => join(root, "src", `${n}.ts`)),
  outdir: out,
  bundle: true,
  format: "esm",
  target: "chrome120",
  logLevel: "info",
});

for (const f of ["manifest.json", "options.html", "icons"]) cpSync(join(root, f), join(out, f), { recursive: true });

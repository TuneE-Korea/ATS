import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Tauri 개발 서버 설정: tauri.conf.json의 devUrl과 포트를 맞춘다.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  // 설정 창(index.html)과 데스크톱 미리보기 창(preview.html) 두 페이지를 빌드한다.
  build: { rollupOptions: { input: { main: "index.html", preview: "preview.html" } } },
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**", "**/crates/**", "**/target/**"] },
  },
});

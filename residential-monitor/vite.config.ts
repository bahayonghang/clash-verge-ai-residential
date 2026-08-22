import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: {
    // 桌面 WebView 从本地加载。默认 500 kB 是 HTTP 站点预算；当前入口约 1 MB。
    chunkSizeWarningLimit: 1024,
    target: "es2022",
    minify: process.env.TAURI_ENV_DEBUG ? false : "esbuild",
    sourcemap: !!process.env.TAURI_ENV_DEBUG
  },
  test: {
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"]
  }
});

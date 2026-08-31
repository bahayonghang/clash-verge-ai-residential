import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

const host = process.env.TAURI_DEV_HOST;

function vendorChunk(id: string): string | undefined {
  const normalized = id.replaceAll("\\", "/");
  if (!normalized.includes("/node_modules/")) {
    return undefined;
  }
  if (
    normalized.includes("/node_modules/react/") ||
    normalized.includes("/node_modules/react-dom/") ||
    normalized.includes("/node_modules/scheduler/")
  ) {
    return "react";
  }
  if (
    normalized.includes("/node_modules/recharts/") ||
    normalized.includes("/node_modules/victory-vendor/") ||
    /\/node_modules\/d3-[^/]+\//.test(normalized)
  ) {
    return "recharts";
  }
  if (normalized.includes("/node_modules/@radix-ui/")) {
    return "radix";
  }
  return undefined;
}

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
    // 路由 lazy + vendor 分包后单块回到 500 kB 守卫。禁止调高阈值掩盖回并。
    chunkSizeWarningLimit: 500,
    target: "es2022",
    minify: process.env.TAURI_ENV_DEBUG ? false : "esbuild",
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    rollupOptions: {
      output: {
        manualChunks: vendorChunk
      }
    }
  },
  test: {
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"]
  }
});

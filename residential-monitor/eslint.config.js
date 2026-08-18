import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    ignores: ["dist/**", "src-tauri/**", "bench-data/**"]
  },
  {
    files: ["src/**/*.ts"],
    rules: {
      "no-eval": "error"
    }
  }
);

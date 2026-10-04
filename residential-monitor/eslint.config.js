import js from "@eslint/js";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import tseslint from "typescript-eslint";

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    ignores: ["dist/**", "src-tauri/**", "bench-data/**"]
  },
  reactHooks.configs.flat.recommended,
  {
    files: ["src/**/*.{ts,tsx}"],
    plugins: {
      react
    },
    settings: {
      react: { version: "detect" }
    },
    rules: {
      "no-eval": "error",
      "react/react-in-jsx-scope": "off",
      "react/jsx-uses-react": "off",
      // Tauri bootstrap and latest-ref IPC patterns set state in effects and write refs during render.
      "react-hooks/set-state-in-effect": "off",
      "react-hooks/refs": "off"
    }
  }
);

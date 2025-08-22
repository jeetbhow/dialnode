import tseslint from "@electron-toolkit/eslint-config-ts";
import eslintConfigPrettier from "@electron-toolkit/eslint-config-prettier";
import eslintPluginSvelte from "eslint-plugin-svelte";

export default tseslint.config(
  { ignores: ["**/node_modules", "**/dist", "**/out"] },
  tseslint.configs.recommended,
  eslintPluginSvelte.configs["flat/recommended"],
  {
    files: ["**/*.svelte"],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser
      }
    }
  },
  {
    files: ["**/*.{ts,tsx,svelte}"],
    rules: {
      "svelte/no-unused-svelte-ignore": "off",
      "no-unused-vars": "off",
      "@typescript-eslint/no-unused-vars": "off",
      "linebreak-style": ["warn", "windows"]
    }
  },
  {
    files: ["**/*.svelte.ts"],
    languageOptions: {
      parser: tseslint.parser
    }
  },
  eslintConfigPrettier
);

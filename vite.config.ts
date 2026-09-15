import { defineConfig } from "vite-plus";

export default defineConfig({
  staged: {
    "*": "vp check --fix",
    "*.rs": () => "cargo fmt --all",
  },
  fmt: {
    ignorePatterns: ["**/*.gen.ts", "packages/contract/openapi.json"],
    sortImports: true,
    sortPackageJson: true,
    sortTailwindcss: {
      functions: ["cn", "cva", "clsx"],
      stylesheet: "apps/web/src/styles.css",
    },
  },
  lint: {
    ignorePatterns: ["**/*.gen.ts"],
    jsPlugins: [{ name: "vite-plus", specifier: "vite-plus/oxlint-plugin" }],
    plugins: ["eslint", "typescript", "react", "jsx-a11y", "import", "promise", "unicorn", "oxc"],
    categories: {
      correctness: "error",
      suspicious: "error",
      perf: "error",
    },
    rules: {
      "vite-plus/prefer-vite-plus-imports": "error",
      "react/react-in-jsx-scope": "off",
      eqeqeq: ["error", "always"],
      "import/no-cycle": "error",
      "no-console": "warn",
      "typescript/no-explicit-any": "error",
      "typescript/no-unsafe-type-assertion": "off",
      "typescript/no-floating-promises": "error",
    },
    options: { typeAware: true, typeCheck: true },
  },
  run: {
    cache: true,
  },
});

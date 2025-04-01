import globals from "globals";

import pluginJs from "@eslint/js";

import tseslint from "typescript-eslint";

import pluginVue from "eslint-plugin-vue";

/** @type {import('eslint').Linter.Config[]} */

export default [
  // 忽略配置

  {
    ignores: [
      "**/dist/*",

      "**/build/*",

      "**/node_modules/*",

      "**/*.d.ts",

      "**/src-tauri/*",

      "**/public/*",

      "**/.vscode/*"
    ]
  },

  // 全局语言环境和全局变量

  {
    languageOptions: {
      globals: {
        ...globals.browser,

        ...globals.node
      },

      // 指定 ECMAScript 版本

      ecmaVersion: "latest",

      sourceType: "module"
    }
  },

  // JavaScript 基础推荐配置

  pluginJs.configs.recommended,

  // TypeScript 推荐配置

  ...tseslint.configs.recommended,

  // Vue 基础配置

  ...pluginVue.configs["flat/essential"],

  // 全局规则配置

  {
    files: ["**/*.{js,mjs,cjs,ts,vue,tsx}"],

    rules: {
      // TypeScript 特定规则
      "@typescript-eslint/no-unused-expressions": [
        "error",

        {
          allowShortCircuit: true,

          allowTernary: true,

          allowTaggedTemplates: false
        }
      ],

      // 允许使用 any 类型
      "@typescript-eslint/no-explicit-any": "off", // 允许使用 any

      // 在生产环境中禁止使用 console，开发环境中允许
      "no-console": process.env.NODE_ENV === "production" ? "warn" : "off",

      // 在生产环境中禁止使用 debugger，开发环境中允许
      "no-debugger": process.env.NODE_ENV === "production" ? "warn" : "off",

      // 允许组件名称为单个单词，关闭 Vue 中多单词组件名称的强制规则
      "vue/multi-word-component-names": "off",

      // 禁止使用 eval 函数，防止潜在的安全漏洞
      "no-eval": "error",

      // 在布尔表达式中禁止多余的强制类型转换，例如 !! 用于布尔值转换
      "no-extra-boolean-cast": "warn",

      // 禁止在代码中出现多个连续的空格，以保持代码格式整洁
      "no-multi-spaces": "error",

      // 强制在导入 TypeScript 类型时使用 'import type' 语法，提高类型导入的一致性
      "@typescript-eslint/consistent-type-imports": [
        "error",

        { prefer: "type-imports" }
      ],

      "vue/no-v-html": "off",

      // 强制使用箭头函数的括号，即使函数体只有一个参数
      "arrow-parens": ["error", "always"],

      "@typescript-eslint/no-duplicate-enum-values": "off",

      // 允许在 catch 块中使用空的代码块
      "no-empty": [
        "error",
        {
          allowEmptyCatch: true
        }
      ]
    }
  },

  // 生产环境规则，将在构建时应用

  {
    files: ["**/*.prod.js"], // 指定生产环境的文件后缀

    rules: {
      "no-console": "error", // 生产环境禁止使用 console

      "no-debugger": "error" // 生产环境禁止使用 debugger
    }
  },

  // Vue 文件特定配置
  {
    files: ["**/*.vue"],

    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,

        extraFileExtensions: [".vue"],
        ecmaFeatures: {
          jsx: true
        }
      }
    },

    rules: {
      // Vue 特定规则
      "vue/multi-word-component-names": "off"
    }
  }
];


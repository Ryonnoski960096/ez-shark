import { defineConfig, Rule } from "unocss";
import presetIcons from "@unocss/preset-icons/browser";
import { flexRules, widthRules, itemRules } from "./src/unocss";
import { presetUno } from "unocss";
export default defineConfig({
  // ...UnoCSS options
  rules: [...flexRules, ...widthRules, ...itemRules] as Rule[],
  presets: [
    presetUno(),
    presetIcons({
      collections: {
        "material-symbols": () =>
          import("@iconify-json/material-symbols/icons.json").then(
            (i) => i.default as any
          ),
        tdesign: () =>
          import("@iconify-json/tdesign/icons.json").then(
            (i) => i.default as any
          ),
        "qlementine-icons": () =>
          import("@iconify-json/qlementine-icons/icons.json").then(
            (i) => i.default as any
          ),
        "icon-park": () =>
          import("@iconify-json/icon-park/icons.json").then(
            (i) => i.default as any
          )
      }
    })
  ]
});

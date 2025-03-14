// src/main.ts
import { createApp } from "vue";
import App from "./App.vue";
import ContextMenu from "@imengyu/vue3-context-menu";
import { createPinia } from "pinia";
import router from "./router";
import { useSettingStore } from "./stores/settings";

import "element-plus/dist/index.css";
import "@/assets/style/index.css";
import "ant-design-vue/dist/reset.css";
// import "@imengyu/vue3-context-menu/lib/vue3-context-menu.css";
import "@/assets/style/vue3-context-menu.css";
import "virtual:uno.css";

async function bootstrap() {
  const app = createApp(App);
  const pinia = createPinia();

  // 初始化全局状态管理
  app.use(pinia);

  const settingStore = useSettingStore();
  await settingStore.init();

  // 注册全局组件和插件
  app.use(ContextMenu);
  app.use(router);

  // 挂载应用
  app.mount("#app").$nextTick(() => {
    postMessage({ payload: "removeLoading" }, "*");
  });
}

// 启动应用
bootstrap().catch((error) => {
  console.error("Application failed to start:", error);
});

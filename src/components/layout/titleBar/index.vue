<!-- TitleBar.vue -->
<template>
  <div @dblclick="handleMaximize" class="title-bar">
    <div class="logo-section">
      <img class="logo" :src="LogoPath" alt="Logo" />
      <span class="title">
        {{ title }}
        {{
          windowManager.isMainWindow()
            ? ` - 监听端口：${settingStore.settings.port}`
            : ""
        }}</span
      >
    </div>
    <WindowControls
      :handleMaximize="handleMaximize"
      v-model="isMaximized"
      :style="controlsStyle"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeMount, ref } from "vue";
import WindowControls from "./WindowControls.vue";
// 后期更改logo
import LogoPath from "/tauri.svg";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { windowManager } from "@/stores/WindowManager";
import { useSettingStore } from "@/stores/settings";

const settingStore = useSettingStore();

const win = getCurrentWebviewWindow();
const isMaximized = ref(false);
// 检查初始状态
win.isMaximized().then((maximized) => {
  isMaximized.value = maximized;
});

const handleMaximize = async () => {
  console.log("双击");
  if (await win.isMaximized()) {
    await win.unmaximize();
    isMaximized.value = false;
  } else {
    await win.maximize();
    isMaximized.value = true;
  }
};

const title = ref("窗口");
onBeforeMount(async () => {
  title.value = await win.title();
});
const props = defineProps<{
  style?: "windows" | "macos";
}>();

const controlsStyle = computed(() => ({
  class: props.style === "macos" ? "macos-style" : "windows-style"
}));
</script>

<style scoped>
.title-bar {
  height: 32px;
  background: rgba(187, 216, 233, 0.479); /* 降低不透明度 */
  backdrop-filter: blur(50px);
  -webkit-backdrop-filter: blur(50px); /* Safari 支持 */
  display: flex;
  justify-content: space-between;
  align-items: center;
  -webkit-app-region: drag;
  padding: 0 0 0 8px;
}

.logo-section {
  display: flex;
  align-items: center;
  gap: 8px;
}

.logo {
  width: 16px;
  height: 16px;
  -webkit-app-region: no-drag;
}

.title {
  font-size: 14px;
  color: #333;
}
</style>

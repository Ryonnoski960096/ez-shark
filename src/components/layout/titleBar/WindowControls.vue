<!-- src/components/WindowControls.vue -->
<template>
  <div @dblclick="(e) => e.preventDefault()" class="window-controls">
    <button
      class="control-button minimize"
      @click="handleMinimize"
      title="最小化"
    >
      <span class="i-qlementine-icons-windows-minimize-16"></span>
    </button>
    <button
      class="control-button maximize"
      @click="handleMaximize"
      title="最大化"
    >
      <span
        v-if="!isMaximized"
        class="i-qlementine-icons-windows-maximize-16"
      />
      <span v-else class="i-qlementine-icons-windows-unmaximize-16" />
    </button>
    <button class="control-button close" @click="handleClose" title="关闭">
      <span class="i-qlementine-icons-windows-close-16"></span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { WindowEvent } from "@/enum/window";
import { useUrlParams } from "@/hooks";

const win = getCurrentWebviewWindow();
const isMaximized = defineModel();
const urlParams = useUrlParams();
const urlParamsObj = urlParams.parseUrlParams();

const { handleMaximize } = defineProps<{
  handleMaximize: () => void;
}>();

// 监听窗口状态变化
win.onResized(() => {
  win.isMaximized().then((maximized) => {
    isMaximized.value = maximized;
  });
});

const handleMinimize = () => {
  if (urlParamsObj.minimize === "false") return;
  win.minimize();
};

const handleClose = () => {
  console.log("请求关闭窗口");
  if (urlParamsObj.windowCloseEnable === "false") return;
  win.emitTo("main", WindowEvent.CLOSE_REQUESTED, win.label);
};
</script>

<style scoped>
.window-controls {
  display: flex;
  -webkit-app-region: no-drag;
  height: 100%;
}

.control-button {
  border: none;
  background: transparent;
  font-size: 16px;
  padding: 0;
  width: 46px;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #666;
  transition:
    background-color 0.2s,
    color 0.2s;
}

.control-button:hover {
  background-color: rgba(0, 0, 0, 0.1);
  color: #000;
}

.control-button.close:hover {
  background-color: #e81123;
  color: white;
}

/* Windows 风格 */
.window-controls.windows-style {
  height: 32px;
}

.window-controls.windows-style .control-button {
  width: 46px;
}

/* macOS 风格 */
.window-controls.macos-style {
  gap: 8px;
  padding: 0 12px;
}

.window-controls.macos-style .control-button {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}

.window-controls.macos-style .close {
  background-color: #ff5f57;
}

.window-controls.macos-style .minimize {
  background-color: #febc2e;
}

.window-controls.macos-style .maximize {
  background-color: #28c840;
}
</style>

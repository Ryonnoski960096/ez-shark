<template>
  <header class="f-l-t header">
    <div class="f-col-center-start w">
      <div style="background-color: #00000008" class="f-col-start-start w">
        <ToolBar class="p-6px" />
        <!-- <div style="height: 1px; width: 100vw; background-color: #00000050" /> -->
      </div>
      <div style="border: 1px solid #00000050" class="w f-l f-g-5 p-2px">
        <template v-for="button in iconButtons" :key="button.titleText">
          <Tooltip :placement="button.placement || 'bottomLeft'">
            <template #title>
              {{ button.titleText }}
            </template>
            <button
              class="cp icon-tool f-l"
              :class="{
                'icon-active': button.isActive
              }"
              @click="button.onClick"
            >
              <span :class="button.icon" />
            </button>
          </Tooltip>
        </template>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { message, Tooltip } from "ant-design-vue";
import ToolBar from "./toolBar.vue";
import { computed, ref } from "vue";
import { useTrafficStore } from "@/stores/traffic";
import type { TooltipPlacement } from "ant-design-vue/es/tooltip";
import { useSettingStore } from "@/stores/settings";
import { useEventBus } from "@/hooks";
import { changeMonitorTraffic } from "@/api/server";
import type { Breakpoints } from "@/hooks/useBreakpointConfig";
import { windowManager } from "@/stores/WindowManager";
import { BreakpointEventName } from "@/enum/breakpoint";
import { deleteTraffic } from "@/api/traffic";
import { error } from "@tauri-apps/plugin-log";
import { useSessionStore } from "@/stores/session";

interface IconButtonConfig {
  icon: string;
  titleText: string; // 改为直接存储文本
  onClick: () => void;
  placement?: TooltipPlacement;
  isActive: boolean; // 改为直接存储布尔值
}

const trafficStore = useTrafficStore();
const sessionStore = useSessionStore();
const settingStore = useSettingStore();
const eventBus = useEventBus();

const breakpoints = ref<Breakpoints | undefined>(
  await settingStore.get("breakpoints")
);

windowManager.window.listen<Breakpoints>(
  BreakpointEventName.BREAKPOINT_CHANGED,
  (data) => {
    breakpoints.value = data.payload;
  }
);
const iconButtons = computed<IconButtonConfig[]>(() => {
  const arr = [
    {
      icon: "i-tdesign-clear",
      titleText: "清除列表",
      onClick: async () => {
        try {
          if (!sessionStore.currentSession) return;
          const currentSessionTraffics = trafficStore.trafficList.get(
            sessionStore.currentSession
          );
          if (!currentSessionTraffics) return;
          // 收集id
          const ids = Array.from(currentSessionTraffics.keys());
          // 发送删除请求
          await deleteTraffic(ids);
          trafficStore.trafficList.set(sessionStore.currentSession, new Map());
          eventBus.emit("change:trafficDetail", null);
          trafficStore.currentTrafficId.set(sessionStore.currentSession, null);
        } catch (e) {
          error(`删除失败: ${e}`);
          message.error(`删除失败: ${e}`);
        }
      },
      isActive: false
    },
    {
      icon: "i-material-symbols-filter-tilt-shift-rounded",
      titleText: trafficStore.isListenerMode ? "暂停监听" : "恢复监听",
      onClick: async () => {
        trafficStore.isListenerMode = !trafficStore.isListenerMode;
        await settingStore.set("isListenerMode", trafficStore.isListenerMode);
        if (trafficStore.isListenerMode) {
          sessionStore.currentListenSession = sessionStore.currentSession;
          await changeMonitorTraffic(sessionStore.currentSession ?? "");
        } else {
          sessionStore.currentListenSession = "";
          await changeMonitorTraffic("");
        }
      },
      isActive: trafficStore.isListenerMode
    },
    {
      icon: "i-material-symbols-conveyor-belt-outline-rounded",
      titleText: "自动滚动",
      onClick: () => {
        trafficStore.isAutoScroll = !trafficStore.isAutoScroll;
        settingStore.set("isAutoScroll", trafficStore.isAutoScroll);
      },
      isActive: trafficStore.isAutoScroll
    }
  ];
  if (breakpoints.value) {
    arr.push({
      icon: breakpoints.value?.toolEnabled
        ? "i-material-symbols-hand-gesture-outline"
        : "i-material-symbols-hand-gesture-off-outline",
      titleText: breakpoints.value?.toolEnabled ? "关闭断点" : "开启断点",
      onClick: () => {
        if (breakpoints.value) {
          breakpoints.value.toolEnabled = !breakpoints.value.toolEnabled;
          settingStore.set("breakpoints", breakpoints.value);
        }
      },
      isActive: breakpoints.value?.toolEnabled
    });
  }
  return arr;
});
</script>

<style scoped>
.icon-tool {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  padding: 6px;
  color: #666;
  width: 32px;
  height: 32px;
  border-radius: 4px;
  transition: all 0.1s cubic-bezier(0.645, 0.045, 0.355, 1);
  border: none;
}

.icon-tool:not(.icon-active):hover {
  color: #1890ff;
  border: 1px solid #0778d4a4;
}

.icon-active {
  background-color: #cce8ff;
  border: 1px solid #0778d4a4;
  color: #de4f8f;
}
</style>

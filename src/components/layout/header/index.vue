<template>
  <header class="f-l-t header p-5px">
    <div class="f-b w">
      <div class="f-c f-g-5">
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
      <div class="f-l p-6px">
        <ToolBar />
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { Tooltip } from "ant-design-vue";
import ToolBar from "./toolBar.vue";
import { computed } from "vue";
import { useTrafficStore } from "@/stores/traffic";
import { TooltipPlacement } from "ant-design-vue/es/tooltip";
import { useSettingStore } from "@/stores/settings";
import { useEventBus } from "@/hooks";
import { changeMonitorTraffic } from "@/api/server";

interface IconButtonConfig {
  icon: string;
  titleText: string; // 改为直接存储文本
  onClick: () => void;
  placement?: TooltipPlacement;
  isActive: boolean; // 改为直接存储布尔值
}

const trafficStore = useTrafficStore();
const settingStore = useSettingStore();
const eventBus = useEventBus();

// 使用 computed 来创建响应式的按钮配置
const iconButtons = computed<IconButtonConfig[]>(() => [
  {
    icon: "i-tdesign-clear",
    titleText: "清除列表",
    onClick: () => {
      trafficStore.trafficList = new Map();
      eventBus.emit("change:trafficDetail", null);
      trafficStore.currentTrafficId = null;
    },
    isActive: false
  },
  {
    icon: "i-material-symbols-filter-tilt-shift-rounded",
    titleText: trafficStore.isListenerMode ? "暂停监听" : "恢复监听",
    onClick: async () => {
      trafficStore.isListenerMode = !trafficStore.isListenerMode;
      await settingStore.set("isListenerMode", trafficStore.isListenerMode);
      changeMonitorTraffic(trafficStore.isListenerMode);
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
]);
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
  transition: all 0.3s cubic-bezier(0.645, 0.045, 0.355, 1);
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

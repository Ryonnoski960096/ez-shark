<template>
  <nav class="tabs f-l f-g-2" :class="size">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      @click="activeTab = tab.id"
      @click.right="onRightClick($event, tab.id)"
      class="tab-item"
      :class="[{ 'tab-active': activeTab === tab.id }, size]"
      type="button"
    >
      {{ tab.label }}
    </button>
  </nav>
</template>

<script setup lang="ts">
import type { Tab } from "./model";

const { tabs, size = "default" } = defineProps<{
  tabs: Tab[];
  size?: "small" | "default";
}>();

const emit = defineEmits<{
  (e: "rightClick", event: MouseEvent, id: string): void;
}>();

const onRightClick = (event: MouseEvent, id: string) => {
  event.preventDefault();
  event.stopPropagation();
  emit("rightClick", event, id);
};

const activeTab = defineModel<string>();
</script>

<style scoped>
.tabs {
  width: 100%;
  background-color: #f2f2f2;
}

/* 默认尺寸 */
.tab-item {
  padding: 4px 10px;
  font-size: 14px;
  cursor: pointer;
  border: none;
  background: transparent;
  transition: all 0.3s ease;
  position: relative;
  height: 32px;
  line-height: 24px;
}

/* 小尺寸 */
.tab-item.small {
  padding: 0px 8px;
  font-size: 12px;
  height: 26px;
  line-height: 20px;
}

.tab-active {
  border: 1px solid #ccc;
  background-color: #fff;
  border-bottom: none;
}

/* tabs 容器尺寸 */
.tabs.default {
  height: 32px;
}

.tabs.small {
  height: 26px;
}
</style>

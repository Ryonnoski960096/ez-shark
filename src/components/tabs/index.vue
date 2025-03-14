<template>
  <nav class="tabs f-l f-g-2">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      @click="activeTab = tab.id"
      class="tab-item"
      :class="{ 'tab-active': activeTab === tab.id }"
      type="button"
    >
      {{ tab.label }}
    </button>
  </nav>
</template>

<script setup lang="ts">
import type { Tab } from "./model";

const { tabs } = defineProps<{
  tabs: Tab[];
}>();

const activeTab = defineModel<number>();
</script>

<style scoped>
.tabs {
  width: 100%;
  background-color: #f2f2f2;
}

.tab-item {
  padding: 5px 14px;
  cursor: pointer;
  border: none;
  background: transparent;
  transition: all 0.3s ease;
  position: relative;
}

.tab-active {
  border: 1px solid #ccc;
  background-color: #fff;
  border-bottom: none;
}
</style>

<template>
  <ul class="list">
    <li
      v-for="[key, value] in items"
      :key="key"
      :class="{ selected: selectedIndex === key }"
      @click="selectItem(key, value)"
    >
      <div class="list-item">
        <span
          title="请求"
          class="traffic-type i-icon-park-up-two"
          v-if="value.traffic_type === 'request'"
        />
        <span
          title="响应"
          class="traffic-type i-icon-park-left-two"
          v-else-if="value.traffic_type === 'response'"
        />
        <span
          title="重发"
          class="traffic-type i-icon-park-electronic-pen"
          v-else-if="value.traffic_type === 'resend'"
        />
        <span class="uri-text">{{ value.traffic.uri }}</span>
      </div>
    </li>
  </ul>
</template>

<script setup lang="ts">
import type { TrafficEditData } from "@/stores/traffic";
import { ref, watch } from "vue";

// 定义 props
interface Props {
  items?: Map<string, TrafficEditData>; // 可选的传入数据
  modelValue?: string | null; // 外部传入的选中索引
}

// 使用 withDefaults 设置默认值
const props = withDefaults(defineProps<Props>(), {
  items: () => new Map(),
  modelValue: null
});

// 定义 emits
const emits = defineEmits<{
  (e: "update:modelValue", value: string | null): void;
  (e: "select", value: { key: string; value: TrafficEditData }): void;
}>();

// 选中的索引
const selectedIndex = ref<string | null>(props.modelValue);

// 监听外部传入的 modelValue
watch(
  () => props.modelValue,
  (newValue) => {
    selectedIndex.value = newValue;
  }
);

// 选择列表项
const selectItem = (key: string, value: TrafficEditData) => {
  selectedIndex.value = key;

  // 更新外部 v-model
  emits("update:modelValue", key);

  // 触发选择事件，返回选中的索引和项目
  emits("select", {
    key,
    value
  });
};
</script>

<style scoped>
.list {
  width: 100%;
  height: 100vh;
  list-style: none;
  font-size: 14px;
  padding: 3px;
  margin: 0;
  text-align: left;
}

.list li {
  padding: 4px 8px;
  cursor: pointer;
  transition: background-color 0.1s ease;
}

.list li:hover {
  background-color: #f0f0f0;
}

.list li.selected {
  background-color: #1890ff;
  color: white;
}

.list-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.traffic-type {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.uri-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}
</style>

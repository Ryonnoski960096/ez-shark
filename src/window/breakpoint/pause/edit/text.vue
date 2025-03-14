<template>
  <div class="h-100% w">
    <Textarea
      class="custom-textarea"
      :readOnly="readOnly"
      style="height: 100%"
      v-model:value="value"
    />
  </div>
</template>

<script lang="ts" setup>
import { Textarea } from "ant-design-vue";

const value = defineModel<string>("value");

const { readOnly = false } = defineProps<{
  readOnly?: boolean;
}>();
</script>

<style scoped>
.custom-textarea {
  /* 重置所有默认样式 */
  all: unset;

  /* 确保继承父容器高度 */
  width: 100%;
  height: 100%;

  /* 重新添加必要的 textarea 样式 */
  display: block;
  resize: none;
  box-sizing: border-box;
  outline: none;
  border: 1px solid #ccc;
  border-radius: 4px;
  padding: 5px;
  margin: 0;
  font-family: inherit;
  font-size: inherit;
  line-height: inherit;
  color: inherit;
  background: #00000008;
}

/* 去除默认的焦点状态 */
.custom-textarea:focus {
  outline: none;
  box-shadow: none;
}
</style>

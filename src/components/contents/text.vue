<template>
  <div class="h-100% w">
    <div
      @input.prevent
      @beforeinput.prevent
      @paste.prevent
      contenteditable="true"
      v-if="readOnly"
      class="readonly-content"
      style="font-size: 12px"
      v-html="readOnlyValue"
    />
    <Textarea
      v-else
      class="custom-textarea"
      :readOnly="readOnly"
      style="height: 100%"
      v-model:value="content"
    />
  </div>
</template>

<script lang="ts" setup>
import { nextTick, ref, watchEffect } from "vue";
import type { ActiveTraffic } from "@/utils/eventBus";
import { escapeHtml, hexToRgba } from "@/utils/format";
import { Textarea } from "ant-design-vue";

const content = defineModel<string>("content", {
  required: true,
  default: ""
});
const { readOnly = false } = defineProps<{
  readOnly?: boolean;
}>();

type MarkStyleOptions = {
  backgroundColor?: string; // 背景色
  borderColor?: string; // 边框色
  borderRadius?: number; // 边角半径
  opacity?: number; // 透明度
};

const generateMarkStyle = (options: MarkStyleOptions) => {
  const {
    backgroundColor = "#FFEB3B", // 默认背景色为淡黄色
    borderColor = "transparent", // 边框颜色
    borderRadius = 8, // 默认圆角
    opacity = 0.4 // 默认不透明度
  } = options;

  // 创建 RGBA 颜色字符串
  const rgbaColor = backgroundColor.startsWith("#")
    ? hexToRgba(backgroundColor, opacity)
    : backgroundColor;

  return `  
    background-color: ${rgbaColor};  
    border: 1px solid ${borderColor};  
    border-radius: ${borderRadius}px;  
    color: #333;  
    padding: 0px 2px;  
    font-weight: 800;  
    font-style: italic;  
    transition: background 0.3s ease;  
  `
    .replace(/\n/g, " ")
    .replace(/\s+/g, " ")
    .trim(); // 规范化样式字符串
};

const readOnlyValue = ref();
watchEffect(() => {
  readOnlyValue.value = escapeHtml(content.value);
});

const highlight = (activeTraffic: ActiveTraffic) => {
  const start = escapeHtml(content.value?.slice(0, activeTraffic.index) ?? "");
  const end = escapeHtml(
    content.value?.slice(activeTraffic.index + activeTraffic.keyword.length) ??
      ""
  );

  readOnlyValue.value =
    start +
    `<mask style='${generateMarkStyle({})}'>${activeTraffic.keyword}</mask>` +
    end;

  nextTick(() => {
    const maskElement = document.querySelector("mask");
    if (maskElement) {
      // 平滑滚动到高亮元素
      maskElement.scrollIntoView({
        behavior: "smooth",
        block: "center"
      });
    }
  });
};

defineExpose({
  highlight
});
</script>

<style scoped>
.custom-textarea {
  /* 之前的样式保持不变 */
  all: unset;
  width: 100%;
  height: 100%;
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

.custom-textarea:focus {
  outline: none;
  box-shadow: none;
}

.readonly-content {
  width: 100%;
  height: 100%;
  /* border: 1px solid #ccc; */
  /* border-radius: 4px; */
  padding: 5px;
  white-space: pre-wrap;
  word-wrap: break-word;
  overflow: auto;
}
</style>

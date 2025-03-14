<template>
  <div class="h-100% w">
    <JsonEditorVue
      ref="jsonEditorVueRef"
      :readOnly="readOnly"
      v-model="value"
      v-bind="JsonEditorVueProps"
    />
  </div>
</template>

<script lang="ts" setup>
import JsonEditorVue from "json-editor-vue";

const value = defineModel<string>("value", {
  required: true,
  default: ""
});

const { readOnly = false } = defineProps<{
  readOnly?: boolean;
}>();

enum Mode {
  text = "text",
  tree = "tree",
  table = "table"
}

const JsonEditorVueProps = {
  mode: Mode.text,
  mainMenuBar: false,
  statusBar: false,
  navigationBar: false,
  style: {
    height: "100%"
  }
};
</script>

<style scoped></style>

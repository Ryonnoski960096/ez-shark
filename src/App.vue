<template>
  <Suspense>
    <div class="h-100%">
      <TitleBar />
      <router-view />
    </div>
  </Suspense>
</template>
<script setup lang="ts">
import TitleBar from "@/components/layout/titleBar/index.vue";
import { windowManager } from "./stores/WindowManager";
import { onBeforeUnmount, onMounted } from "vue";

// 处理全局搜索
const handleKeydown = async (event: KeyboardEvent) => {
  if (event.ctrlKey && event.key === "f") {
    event.preventDefault(); // 防止默认的 Ctrl + F 行为
    if (!windowManager.isMainWindow()) return;
    // 打开搜索窗口
    await windowManager.createWindow(
      {
        url: "/search"
      },
      {
        title: "全局搜索",
        width: 900,
        height: 700
      }
    );
  }
};

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>
<style>
.app {
  background-color: #f2f2f2;
  padding: 5px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  height: calc(100vh);
}
</style>

<template>
  <div v-if="renderEnable" class="index">
    <Header />
    <Main class="main" />
  </div>
</template>

<script setup lang="ts">
import { setPort } from "@/api/server";
import Header from "@/components/layout/header/index.vue";
import Main from "@/components/layout/main/index.vue";
import { PortEvent } from "@/enum/port";
import { windowManager } from "@/stores/WindowManager";
import { useSettingStore } from "@/stores/settings";
import { useTrafficStore } from "@/stores/traffic";
import { message } from "@tauri-apps/plugin-dialog";
import { onMounted, onUnmounted, ref } from "vue";

const renderEnable = ref(false);
const settingStore = useSettingStore();

const portInit = async (port: number) => {
  try {
    await setPort(port);
    await settingStore.set("port", port);
    await windowManager.window.setEnabled(true);
    renderEnable.value = true;
    await windowManager.window.setFocus();
    // 开始监听流量变化
  } catch (error) {
    await message(("端口设置失败：" + error) as string, { kind: "error" });
    const [wvw] = await windowManager.createWindow(
      {
        url: "/setting/port",
        param: {
          minimize: false,
          windowCloseEnable: false
        }
      },
      {
        title: "端口设置",
        width: 300,
        height: 150
      }
    );

    wvw.once(PortEvent.SUBMIT, async (event) => {
      const port = event.payload as number;
      windowManager.requestClose(wvw);
      await portInit(port);
    });
  }
};

const init = async () => {
  console.log(await windowManager.window.setEnabled(false));
  try {
    // 初始化设置
    await settingStore.portInit();
    // 初始化端口
    await portInit(settingStore.settings.port);

    // 初始化断点
    await settingStore.initBreakpoint();

    await settingStore.monitorTrafficInit();
  } catch (error) {
    console.error("初始化失败:", error);
    throw error;
  }
};

init();

const trafficStore = useTrafficStore();

// 在组件挂载时设置监听
onMounted(() => {
  trafficStore.setupTrafficMonitor();
});

// 在组件卸载时清理
onUnmounted(() => {
  trafficStore.clearListen();
});
</script>

<style scoped>
.index {
  background-color: #f2f2f2;
  height: calc(100% - 74px);
}
</style>

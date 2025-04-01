<template>
  <div v-if="renderEnable" class="index">
    <Header />
    <Tabs
      :tabs="tabList"
      v-model="sessionStore.currentSession"
      @rightClick="tabOnRightClick"
      size="small"
      class="mt-2px"
    />
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
import { error } from "@tauri-apps/plugin-log";
import { message } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, onUnmounted, ref } from "vue";
import Tabs from "@/components/tabs/index.vue";
import { useSessionStore } from "@/stores/session";
import ContextMenu from "@imengyu/vue3-context-menu";

const renderEnable = ref(false);
const settingStore = useSettingStore();
const sessionStore = useSessionStore();

const tabList = computed(() => {
  return sessionStore.sessionList.map((item) => {
    return {
      label:
        sessionStore.currentListenSession === item.id
          ? item.label + " *"
          : item.label,
      id: item.id
    };
  });
});

const tabOnRightClick = (e: MouseEvent, id: string) => {
  e.preventDefault();
  ContextMenu.showContextMenu({
    x: e.clientX,
    y: e.clientY,
    items: [
      {
        label: "删除 Session",
        onClick: () => {
          sessionStore.removeSession(id);
        }
      },
      {
        label: "删除所有 Session",
        onClick: () => {
          sessionStore.removeAllSession();
        }
      }
    ]
  });
};

const portInit = async (port: number) => {
  try {
    await setPort(port);
    await settingStore.set("port", port);
    await windowManager.window.setEnabled(true);
    renderEnable.value = true;
    await windowManager.window.setFocus();
    // 开始监听流量变化
  } catch (e) {
    await message(("端口设置失败：" + e) as string, { kind: "error" });
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
  try {
    // 初始化设置
    await settingStore.portInit();
    // 初始化端口
    await portInit(settingStore.settings.port);

    // 初始化断点
    await settingStore.initBreakpoint();

    // 初始化 Session
    await settingStore.initSession();

    // 初始化外挂代理
    await settingStore.externalProxyInit();

    await settingStore.monitorTrafficInit();
  } catch (e) {
    error("初始化失败:" + e);
    throw e;
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
  height: calc(100% - 124px);
}
</style>

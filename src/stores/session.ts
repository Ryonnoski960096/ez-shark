import type { Tab } from "@/components/tabs/model";
import { defineStore } from "pinia";
import { ref, watch } from "vue";
import { useSettingStore } from "./settings";
import { changeMonitorTraffic } from "@/api/server";
import { getFileNameWithoutExt } from "@/utils/file";
// 定义 Session 接口
export const useSessionStore = defineStore("session", () => {
  const settingsStore = useSettingStore();
  // 定义 Session 数据
  const currentSession = ref<string>();
  // 定义 Session 列表(tabs)
  const sessionList = ref<Tab[]>([]);
  // 定义 当前监听的session
  const currentListenSession = ref<string>();
  // 添加session
  const addSession = (tab: Tab) => {
    sessionList.value.push(tab);
    currentSession.value = tab.id;
  };
  // 删除session
  const removeSession = (id: string) => {
    // 如果只有一个了就不执行
    if (sessionList.value.length === 1) return;

    const index = sessionList.value.findIndex((tab) => tab.id === id);
    if (index !== -1) {
      sessionList.value.splice(index, 1);
      currentSession.value = sessionList.value[index - 1].id;
    }
  };

  // 删除所有session
  const removeAllSession = () => {
    sessionList.value = [
      {
        id: "1",
        label: "Session 1"
      }
    ];
    currentSession.value = "1";
    currentListenSession.value = "1";
    changeMonitorTraffic("1");
  };

  const createSessionForPath = (path: string) => {
    const sessionName = getFileNameWithoutExt(path);
    const sessionStore = useSessionStore();
    const id = Date.now().toString();
    sessionStore.addSession({
      id,
      label: sessionName
    });
    return id;
  };

  watch(currentListenSession, async (newValue) => {
    settingsStore.set("currentListenSession", newValue);
  });

  // 自动保存
  watch(currentSession, (newValue) => {
    settingsStore.set("currentSession", newValue);
  });

  watch(
    sessionList,
    (newValue) => {
      settingsStore.set("sessionList", newValue);
    },
    {
      deep: 1
    }
  );

  return {
    currentSession,
    sessionList,
    currentListenSession,
    addSession,
    removeSession,
    removeAllSession,
    createSessionForPath
  };
});

import { defineStore } from "pinia";
import { ref } from "vue";
import { Store } from "@tauri-apps/plugin-store";
import { windowManager } from "./WindowManager";
import { useTrafficStore } from "./traffic";
import { invoke } from "@tauri-apps/api/core";
import { defaultData as defaultExternalProxyData } from "@/window/externalProxy/model";
import { debug, error } from "@tauri-apps/plugin-log";
import { useSessionStore } from "./session";
import { changeMonitorTraffic } from "@/api/server";

// 定义设置接口
export const useSettingStore = defineStore("setting", () => {
  const trafficStore = useTrafficStore();
  const sessionStore = useSessionStore();

  const store = ref<Promise<Store>>(
    Store.load("settings.json", {
      autoSave: true
    })
  );
  const settings = ref<Record<string, any>>({});
  /**
   * 初始化断点配置
   * 这个方法必须在init之后
   */
  const initBreakpoint = async () => {
    if (!settings.value.breakpoints) {
      settings.value.breakpoints = {
        breakpoints: {},
        toolEnabled: true
      };

      await set("breakpoints", settings.value.breakpoints);
    }
  };

  /**
   * 初始化 Session
   * 这个方法必须在init之后
   */
  const initSession = async () => {
    if (!settings.value.currentSession) {
      settings.value.currentSession = "1";
      await set("currentSession", settings.value.currentSession);
    }
    if (
      !settings.value.sessionList ||
      settings.value.sessionList.length === 0
    ) {
      settings.value.sessionList = [
        {
          id: "1",
          label: "Session 1"
        }
      ];
      await set("sessionList", settings.value.sessionList);
    }

    if (!settings.value.currentListenSession) {
      await set("currentListenSession", "");
    }

    sessionStore.sessionList = settings.value.sessionList;
    sessionStore.currentSession = settings.value.currentSession;

    sessionStore.currentListenSession = settings.value.currentListenSession;
    await changeMonitorTraffic(sessionStore.currentListenSession!);
  };

  /**
   * 初始化端口
   * 这个方法必须在init之后
   */
  const portInit = async () => {
    if (!windowManager.isMainWindow()) return;
    const port = (settings.value.port ??= 8081);
    await set("port", port);
  };

  /**
   * 服务init
   * 这个方法必须在初始化端口之后
   */
  const monitorTrafficInit = async () => {
    return invoke("change_monitor_traffic", {
      monitorTraffic: sessionStore.currentSession
    });
  };

  /**
   * 外挂代理init
   * 这个方法必须在init之后
   */
  const externalProxyInit = async () => {
    if (
      settings.value.externalProxy &&
      settings.value.externalProxy.bypassDomains
    )
      return;
    await set("externalProxy", defaultExternalProxyData);
  };

  /**
   * 初始化设置
   */
  const init = async () => {
    try {
      const s = await store.value;
      const entries = await s.entries();
      for (const [key, value] of entries) {
        settings.value[key] = value;
      }
      debug("settings.value:", settings.value);
      console.log("settings:", settings.value);

      trafficStore.isAutoScroll = settings.value.isAutoScroll ?? false;
      trafficStore.isListenerMode = settings.value.isListenerMode ?? false;
    } catch (e) {
      error("Failed to initialize store.value:" + e);
      throw e;
    }
  };

  const set = async (key: string, value: any) => {
    if (!store.value) {
      throw new Error("Store is not initialized");
    }

    try {
      const s = await store.value;
      await s.set(key, value);
      settings.value[key] = value;
    } catch (e) {
      error(`Failed to set ${key}:${e}`);
      throw e;
    }
  };

  const get = async <T>(key: string): Promise<T | undefined> => {
    if (!store.value) {
      throw new Error("Store is not initialized");
    }

    try {
      return settings.value[key];
    } catch (e) {
      error(`Failed to get ${key}:${e}`);
      throw e;
    }
  };

  const remove = async (key: string) => {
    if (!store.value) {
      throw new Error("Store is not initialized");
    }

    try {
      const s = await store.value;
      await s.delete(key);
      delete settings.value[key];
    } catch (e) {
      error(`Failed to remove ${key}:${e}`);
      throw e;
    }
  };

  const clear = async () => {
    if (!store.value) {
      throw new Error("Store is not initialized");
    }

    try {
      const s = await store.value;
      await s.clear();
      settings.value = {};
    } catch (e) {
      error("Failed to clear store.value:" + e);
      throw e;
    }
  };

  const getAll = (): Record<string, any> => {
    return settings.value;
  };

  return {
    store,
    settings,
    externalProxyInit,
    monitorTrafficInit,
    portInit,
    initBreakpoint,
    initSession,
    init,
    set,
    get,
    remove,
    clear,
    getAll
  };
});

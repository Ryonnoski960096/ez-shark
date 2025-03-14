import { defineStore } from "pinia";
import { ref } from "vue";
import { Store } from "@tauri-apps/plugin-store";
import { updateBreakpoint } from "@/api/breakpoint";
import { windowManager } from "./WindowManager";
import { useTrafficStore } from "./traffic";
import { invoke } from "@tauri-apps/api/core";

// 定义设置接口
export const useSettingStore = defineStore("setting", () => {
  const trafficStore = useTrafficStore();

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

    if (windowManager.isMainWindow()) {
      const breakpointList = [];
      for (const key in settings.value.breakpoints.breakpoints) {
        breakpointList.push(settings.value.breakpoints.breakpoints[key]);
      }

      await updateBreakpoint(breakpointList);
    }
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
      monitorTraffic: trafficStore.isListenerMode
    });
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
      console.log("settings.value:", settings.value);

      trafficStore.isAutoScroll = settings.value.isAutoScroll ?? false;
      trafficStore.isListenerMode = settings.value.isListenerMode ?? false;
    } catch (error) {
      console.error("Failed to initialize store.value:", error);
      throw error;
    }
  };

  const set = async (key: string, value: any) => {
    if (!store.value) {
      throw new Error("Store is not initialized");
    }

    try {
      const s = await store.value;
      await s.set(key, value);
      // console.log(`Set ${key} to `, value);
      settings.value[key] = value;
    } catch (error) {
      console.error(`Failed to set ${key}:`, error);
      throw error;
    }
  };

  const get = async <T>(key: string): Promise<T | undefined> => {
    if (!store.value) {
      throw new Error("Store is not initialized");
    }

    try {
      return settings.value[key];
    } catch (error) {
      console.error(`Failed to get ${key}:`, error);
      throw error;
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
    } catch (error) {
      console.error(`Failed to remove ${key}:`, error);
      throw error;
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
    } catch (error) {
      console.error("Failed to clear store.value:", error);
      throw error;
    }
  };

  const getAll = (): Record<string, any> => {
    return settings.value;
  };

  return {
    store,
    settings,
    monitorTrafficInit,
    portInit,
    initBreakpoint,
    init,
    set,
    get,
    remove,
    clear,
    getAll
  };
});

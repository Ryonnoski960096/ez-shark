import type { WindowManager } from "@/stores/WindowManager";
import { windowManager } from "@/stores/WindowManager";
import { error } from "@tauri-apps/plugin-log";

const handleProxyAction = async (
  ops: Parameters<typeof WindowManager.prototype.createWindow>
) => {
  try {
    const win = windowManager;
    win.createWindow(...ops);
  } catch (e) {
    error("创建子窗口失败:" + e);
  }
};

export const proxyMenuItems = [
  {
    label: "Breakpoint Settings",
    action: "breakpoint",
    click: () =>
      handleProxyAction([
        {
          url: "/breakpoint"
        },
        {
          width: 800,
          height: 600,
          title: "断点配置"
        }
      ])
  },
  {
    label: "External Proxy Settings",
    action: "externalProxy",
    click: () =>
      handleProxyAction([
        {
          url: "/externalProxy"
        },
        {
          width: 800,
          height: 680,
          title: "外部代理配置"
        }
      ])
  }
];

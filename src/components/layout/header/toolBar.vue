<template>
  <div class="toolbar f-r f-g-20">
    <DropdownMenu
      v-for="menu in dropdownMenus"
      :key="menu.title"
      :title="menu.title"
      :menuItems="menu.menuItems"
    />
  </div>
</template>

<script setup lang="ts">
import DropdownMenu from "@/components/dropdownMenu/index.vue";
import { MenuItem } from "@/components/dropdownMenu/model";
import { useExport, useImport } from "@/hooks";
import { WindowManager, windowManager } from "@/stores/WindowManager";
import { installCertificate } from "@/api/help";
import { message } from "ant-design-vue";
import { setPort } from "@/api/server";
import { useSettingStore } from "@/stores/settings";
import { PortEvent } from "@/enum/port";
import { useTrafficStore } from "@/stores/traffic";
import { commonIE } from "@/utils/tools";

const exportTool = useExport();

const importTool = useImport();
const trafficStore = useTrafficStore();

const importMenuItems = [
  {
    label: "Import from Session",
    action: "session",
    click: () => {
      commonIE(async () => {
        const traffics = await importTool.importSession();
        const map = new Map(traffics.map((traffic) => [traffic.id, traffic]));
        trafficStore.trafficList = map;
      }, "导入");
    }
  }
];

const settingStore = useSettingStore();

const exportMenuItems: MenuItem[] = [
  {
    label: "Export as all Markdown",
    action: "markdown",
    click: () => commonIE(exportTool.exportMarkdown)
  },
  {
    label: "Export as all cURL",
    action: "curl",
    click: () => commonIE(exportTool.exportCurl)
  },
  {
    label: "Export as all HAR",
    action: "har",
    click: () => commonIE(exportTool.exportHar)
  },
  {
    label: "Export as all JSON",
    action: "json",
    click: () => commonIE(exportTool.exportJson)
  },
  {
    label: "Export as Session",
    action: "session",
    click: () => commonIE(exportTool.exportJson)
  }
];

const handleProxyAction = async (
  ops: Parameters<typeof WindowManager.prototype.createWindow>
) => {
  try {
    const win = windowManager;
    win.createWindow(...ops);
  } catch (error) {
    console.error("创建子窗口失败:", error);
  }
};

const proxyMenuItems = [
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

const handleSetPort = async () => {
  const [wvw] = await windowManager.createWindow(
    {
      url: "/setting/port"
    },
    {
      title: "端口设置",
      width: 300,
      height: 150
    }
  );

  wvw.once(PortEvent.SUBMIT, async (event) => {
    console.log("PortEvent.SUBMIT", event);
    const port = event.payload as number;
    let hide;
    try {
      hide = message.loading("正在设置端口...");
      await setPort(port);
      hide();
      message.success("端口设置成功");
      settingStore.set("port", port);
      // 开始监听流量变化
      await settingStore.monitorTrafficInit();

      // 设置流量监听器
      // await trafficStore.setupTrafficMonitor();
    } catch (error) {
      hide && hide();
      message.error("端口设置失败：" + error);
    } finally {
      windowManager.requestClose(wvw);
    }
  });
};

const settingsMenuItems = [
  { label: "Set Port", action: "set-port", click: handleSetPort }
];

const helpMenuItems = [
  {
    label: "Install Certificate",
    action: "ICer",
    click: installCertificate
  }
];

const dropdownMenus = [
  {
    title: "Import",
    menuItems: importMenuItems
  },
  {
    title: "Export",
    menuItems: exportMenuItems
  },
  {
    title: "Proxy",
    menuItems: proxyMenuItems
  },
  {
    title: "Settings",
    menuItems: settingsMenuItems
  },
  {
    title: "Help",
    menuItems: helpMenuItems
  }
];
</script>

<style scoped></style>

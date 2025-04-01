import { message } from "ant-design-vue";
import { setPort } from "@/api/server";
import { useSettingStore } from "@/stores/settings";
import { PortEvent } from "@/enum/port";
import { windowManager } from "@/stores/WindowManager";
import { open } from "@tauri-apps/plugin-dialog";
import { commonIE } from "@/utils/tools";

const handleSetPort = async () => {
  const settingStore = useSettingStore();
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
    const port = event.payload as number;
    let hide;
    try {
      hide = message.loading("正在设置端口...", 0);
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

export const handleSetCharlesPath = async () => {
  const settingStore = useSettingStore();
  const path = await open({
    filters: [
      {
        name: "exe",
        extensions: ["exe"]
      }
    ]
  });
  if (!path) throw new Error("未选择文件");
  await settingStore.set("charlesPath", path);
};

export const settingsMenuItems = [
  { label: "Set Port", action: "set-port", click: handleSetPort },
  {
    label: "Set Charles Path",
    action: "set-charles-path",
    click: () => {
      commonIE(handleSetCharlesPath, "设置 Charles 路径");
    }
  }
];

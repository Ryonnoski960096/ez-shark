import { commonIE } from "@/utils/tools";
import { useImport } from "@/hooks";
import { useTrafficStore } from "@/stores/traffic";
import { ElMessageBox } from "element-plus";
import { useSettingStore } from "@/stores/settings";
import { handleSetCharlesPath } from "./settings";
import { isCharlesRunning, killCharles } from "@/api/system";

const importTool = useImport();

export const importMenuItems = [
  {
    label: "Import from Session",
    action: "session",
    click: () => {
      commonIE(async () => {
        const [traffics, sessionId] = await importTool.importSession();

        const trafficStore = useTrafficStore();
        const map = new Map(traffics.map((traffic) => [traffic.id, traffic]));
        trafficStore.trafficList.set(sessionId, map);
      }, "导入");
    }
  },
  {
    label: "Import from Charles",
    action: "charles",
    click: async () => {
      const msgBox = await ElMessageBox.confirm(
        `无法直接导入charles流量，需要先使用charles命令行工具转为har再通过har导入<br/>
        在导入前请确保：<br/><b>* 设置了正确的charles路径</b><br/><b>* charles处于关闭状态</b>`,
        "提示",
        {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning",
          dangerouslyUseHTMLString: true
        }
      );
      if (!msgBox) return;

      const settingStore = useSettingStore();
      const charlesPath = await settingStore.get("charlesPath");
      if (!charlesPath || charlesPath === "") {
        const msgBox = await ElMessageBox.alert("请先设置charles路径", "提示", {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning"
        });
        if (!msgBox) return;
        await handleSetCharlesPath();
      }

      const charlesIsRunning = await isCharlesRunning();
      if (charlesIsRunning) {
        const msgBox = await ElMessageBox.alert(
          "charles正在运行，点击确定强制关闭charles",
          "提示",
          {
            confirmButtonText: "确定",
            type: "error"
          }
        );
        if (!msgBox) return;
        const killStatus = await killCharles();
        if (!killStatus) {
          await ElMessageBox.alert(
            "强制关闭charles失败，请手动关闭charles",
            "提示",
            {
              confirmButtonText: "确定",
              type: "warning"
            }
          );
          return;
        }
      }

      await commonIE(async () => {
        const [traffics, sessionId] = await importTool.importCharles();
        const trafficStore = useTrafficStore();

        const map = new Map(traffics.map((traffic) => [traffic.id, traffic]));
        trafficStore.trafficList.set(sessionId, map);
      }, "导入");
    }
  },
  {
    label: "Import from HAR",
    action: "har",
    click: () => {
      commonIE(async () => {
        const [traffics, sessionId] = await importTool.importHAR();
        const trafficStore = useTrafficStore();

        const map = new Map(traffics.map((traffic) => [traffic.id, traffic]));
        trafficStore.trafficList.set(sessionId, map);
      }, "导入");
    }
  }
];

import { importCharlesApi, importSessionApi, importHARApi } from "@/api/import";
import { useSessionStore } from "@/stores/session";
import type { TrafficData } from "@/stores/traffic";
import { readXmlFile } from "@/utils/file";
import { open } from "@tauri-apps/plugin-dialog";
import { exists } from "@tauri-apps/plugin-fs";

export default function useImport() {
  /**
   * 导入session
   */
  const importSession = async (): Promise<[TrafficData[], string]> => {
    const path = await open({
      filters: [
        {
          name: "JSON",
          extensions: ["json"]
        }
      ]
    });
    if (!path) throw new Error("未选择文件");
    const sessionStore = useSessionStore();
    const sessionId = sessionStore.createSessionForPath(path);
    //
    return [await importSessionApi(path, sessionId), sessionId];
  };

  /**
   * 导入XML
   */
  const importXmlFile = async () => {
    const path = await open({
      title: "导入",
      filters: [{ name: "xml", extensions: ["xml"] }],
      multiple: false
    });
    if (!path || !exists(path)) throw new Error("文件不存在");
    return await readXmlFile(path);
  };

  /**
   * 导入chls
   */
  const importHAR = async (): Promise<[TrafficData[], string]> => {
    const path = await open({
      title: "导入",
      filters: [{ name: "har", extensions: ["har"] }],
      multiple: false
    });
    if (!path || !exists(path)) throw new Error("文件不存在");
    const sessionStore = useSessionStore();
    const sessionId = sessionStore.createSessionForPath(path);
    return [await importHARApi(path, sessionId), sessionId];
  };

  /**
   * 导入chls
   */
  const importCharles = async (): Promise<[TrafficData[], string]> => {
    const path = await open({
      title: "导入",
      filters: [{ name: "chls", extensions: ["chls"] }],
      multiple: false
    });
    if (!path || !exists(path)) throw new Error("文件不存在");
    const sessionStore = useSessionStore();
    const sessionId = sessionStore.createSessionForPath(path);
    return [await importCharlesApi(path, sessionId), sessionId];
  };

  return {
    importSession,
    importXmlFile,
    importCharles,
    importHAR
  };
}

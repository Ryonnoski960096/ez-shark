import { importSessionApi } from "@/api/import";
import { readXmlFile } from "@/utils/file";
import { open } from "@tauri-apps/plugin-dialog";
import { exists } from "@tauri-apps/plugin-fs";

export default function useImport() {
  /**
   * 导入session
   */
  const importSession = async () => {
    const path = await open({
      filters: [
        {
          name: "JSON",
          extensions: ["json"]
        }
      ]
    });
    if (!path) throw new Error("未选择文件");

    return await importSessionApi(path);
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

  return {
    importSession,
    importXmlFile
  };
}

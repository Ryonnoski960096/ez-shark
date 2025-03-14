import { exportApi } from "@/api/export";
import { createXmlStr } from "@/utils/file";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";

export default function useExport() {
  /**
   * 保存为markdown
   */
  const exportMarkdown = async () => {
    const path = await save({
      filters: [
        {
          name: "Markdown",
          extensions: ["md"]
        }
      ]
    });
    if (!path) throw new Error("未选择文件");

    return exportApi(path);
  };

  /**
   * 保存为json
   */
  const exportJson = async () => {
    const path = await save({
      filters: [
        {
          name: "JSON",
          extensions: ["json"]
        }
      ]
    });
    if (!path) throw new Error("未选择文件");

    return exportApi(path);
  };

  /**
   * 保存为har
   */
  const exportHar = async () => {
    const path = await save({
      filters: [
        {
          name: "HAR",
          extensions: ["har"]
        }
      ]
    });
    if (!path) throw new Error("未选择文件");

    return exportApi(path);
  };

  /**
   * 保存为curl
   */
  const exportCurl = async () => {
    const path = await save({
      filters: [
        {
          name: "CURL",
          extensions: ["sh"]
        }
      ]
    });
    if (!path) throw new Error("未选择文件");

    return exportApi(path);
  };

  const exportXML = async (xmlObj: object) => {
    // if (!xmlObj) throw new Error("");
    const path = await save({
      filters: [
        {
          name: "Xml",
          extensions: ["xml"]
        }
      ]
    });
    if (!path) throw new Error("未选择文件");

    return await writeTextFile(path, createXmlStr({ externalProxy: xmlObj }));
  };

  return {
    exportMarkdown,
    exportJson,
    exportHar,
    exportCurl,
    exportXML
  };
}

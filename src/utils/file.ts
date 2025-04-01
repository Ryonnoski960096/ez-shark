import { XMLBuilder, XMLParser } from "fast-xml-parser";
import { readTextFile } from "@tauri-apps/plugin-fs";
import { error } from "@tauri-apps/plugin-log";

export const readXmlFile = async (filePath: string) => {
  try {
    const xmlContent = await readTextFile(filePath);

    // 解析 XML
    const parser = new XMLParser({
      ignoreAttributes: false,
      attributeNamePrefix: "@_",
      parseAttributeValue: true
    });

    const result = parser.parse(xmlContent);
    return result;
  } catch (e) {
    error("Error reading or parsing XML file:" + e);
    throw e;
  }
};

export const createXmlStr = (xmlObj: object) => {
  // 配置 XMLBuilder
  const builder = new XMLBuilder({
    format: true,
    indentBy: "  ",
    ignoreAttributes: false,
    suppressEmptyNode: true
  });

  // 生成 XML 字符串
  return `<?xml version="1.0" encoding="UTF-8"?>\n<?ez-shark serialisation-version='1.0'?>\n${builder.build(xmlObj)}`;
};

export function getFileNameWithoutExt(path: string): string {
  // 获取最后一个斜杠后的内容
  const fileName = path.split(/[\\/]/).pop() || "";
  // 获取最后一个点号前的内容
  const lastDotIndex = fileName.lastIndexOf(".");
  return lastDotIndex === -1 ? fileName : fileName.substring(0, lastDotIndex);
}

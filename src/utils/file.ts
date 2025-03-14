import { XMLBuilder, XMLParser } from "fast-xml-parser";
import { readTextFile } from "@tauri-apps/plugin-fs";

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
  } catch (error) {
    console.error("Error reading or parsing XML file:", error);
    throw error;
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
  return `<?xml version="1.0" encoding="UTF-8"?>\n<?ezshark serialisation-version='1.0'?>\n${builder.build(xmlObj)}`;
};

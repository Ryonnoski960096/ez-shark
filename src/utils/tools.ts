import useUrlParams from "@/hooks/useUrlParams";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { message } from "ant-design-vue";

/**
 * 复制文本到剪贴板的工具函数
 * 此函数为惰性函数，根据浏览器支持情况返回不同的实现函数，并且不会每次都重复判断浏览器是否支持最新API
 * @param text - 要复制的文本
 * @returns 一个函数，当调用时，将文本复制到剪贴板
 */
export const copyText = (function () {
  if (navigator.clipboard) {
    return (text: string) => {
      navigator.clipboard.writeText(text);
    };
  } else {
    return (text: string) => {
      const input = document.createElement("input");
      input.value = text;
      document.body.appendChild(input);
      input.select();
      document.execCommand("Copy");
      document.body.removeChild(input);
    };
  }
})();

export const copyContent = (text: string) => {
  try {
    if (!text) throw new Error("没有内容可以复制");
    copyText(text);
    message.success("复制成功");
  } catch (error) {
    message.error("复制失败");
  }
};

const urlParams = useUrlParams();

/**
 * 获取父窗口id
 * @returns
 */
export const getParentWindowId = () => {
  const parentWindowId = urlParams.getUrlParam("parentWindowId");

  if (!parentWindowId) throw new Error("parentWindowId is null");
  return parentWindowId;
};

/**
 * 深拷贝一个对象或数组
 *
 * @param value - 要克隆的对象或数组
 * @param hash - 用于存储已克隆对象的哈希表，以避免循环引用
 * @returns - 克隆后的对象或数组
 */
export function deepClone(
  value: any,
  hash: WeakMap<any, any> = new WeakMap()
): any {
  // 基本数据类型直接返回
  if (value === null || typeof value !== "object") {
    return value;
  }

  // 如果是日期对象
  if (value instanceof Date) {
    return new Date(value);
  }

  // 如果是正则对象
  if (value instanceof RegExp) {
    return new RegExp(value);
  }

  // 处理函数（注意，函数不能被序列化）
  if (typeof value === "function") {
    return value;
  }

  // 检查循环引用
  if (hash.has(value)) {
    return hash.get(value);
  }

  // 创建一个新对象或数组
  const result: any = Array.isArray(value) ? [] : {};

  // 缓存当前值，以处理循环引用
  hash.set(value, result);

  // 递归克隆每个属性
  Object.keys(value).forEach((key) => {
    if (Object.prototype.hasOwnProperty.call(value, key)) {
      // 确保 result[key] 类型正确
      result[key] = deepClone(value[key], hash);
    }
  });

  return result;
}

/**
 * 下载文件
 * @param content
 * @param fileName
 */
export async function saveWithDialog(content: string) {
  try {
    // 打开保存对话框
    const filePath = await save({
      filters: [
        {
          name: "Text",
          extensions: ["txt"]
        }
      ]
    });

    if (filePath) {
      // 保存文件
      await writeTextFile(filePath, content);
      console.log("File saved successfully");
    }
  } catch (err) {
    console.error("Error saving file:", err);
  }
}

/**
 * 通用的导出导入函数
 * @param fn 导出导入函数
 * @param method
 * @param afterFn 导入或者导出的后置操作
 */
export const commonIE = async (
  fn: () => Promise<unknown>,
  method: "导出" | "导入" = "导出",
  afterFn?: () => void
) => {
  const hide = message.loading(`正在${method}...`);
  try {
    await fn();
    hide();
    afterFn && afterFn();
    message.success(`${method}成功`);
  } catch (error) {
    hide();
    message.error(`${method}失败: ${error}`);
  }
};

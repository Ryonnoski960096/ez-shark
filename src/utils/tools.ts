import useUrlParams from "@/hooks/useUrlParams";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { error } from "@tauri-apps/plugin-log";
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
  } catch {
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
    }
  } catch (err) {
    error("Error saving file:" + err);
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
  method: string = "导出",
  afterFn?: () => void
) => {
  const hide = message.loading(`正在${method}...`, 0);
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

/**
 * 等待条件满足的异步函数
 * @param checkFunction - 一个返回布尔值的函数，用于检查条件是否满足
 * @param interval - 检查的时间间隔（毫秒），默认为 50 毫秒
 * @returns - 一个 Promise，当条件满足时解析
 */
export const waitForCondition = (
  checkFunction: () => boolean,
  interval: number = 50
): Promise<void> => {
  return new Promise((resolve) => {
    const check = () => {
      if (checkFunction()) {
        resolve(); // 如果条件满足，解析 Promise
      } else {
        setTimeout(check, interval); // 否则每隔 interval 毫秒检查一次
      }
    };
    check(); // 初始化检查
  });
};

/**
 * 验证字符串是否是有效的Base64编码
 * @param  str - 要验证的字符串
 * @param  allowPadding - 是否允许包含填充字符'='（默认为true）
 * @param  strictMode - 是否使用严格模式检查（默认为false）
 * @returns  - 如果是有效的Base64编码则返回true，否则返回false
 */
export function isValidBase64(
  str: string,
  allowPadding = true,
  strictMode = false
) {
  // 1. 基本检查：非空字符串
  if (!str || typeof str !== "string") {
    return false;
  }

  // 2. 移除所有空白字符（有些Base64可能被格式化）
  const trimmedStr = str.replace(/\s/g, "");

  // 3. 检查是否为空
  if (trimmedStr.length === 0) {
    return false;
  }

  // 4. 检查长度是否符合Base64规则（必须是4的倍数，若有填充）
  if (strictMode && !allowPadding) {
    // 如果不允许填充，长度可以是任意的
  } else if (strictMode && allowPadding && trimmedStr.length % 4 !== 0) {
    return false;
  }

  // 5. 处理Data URL前缀
  let base64Str = trimmedStr;
  if (trimmedStr.startsWith("data:") && trimmedStr.includes("base64,")) {
    base64Str = trimmedStr.split("base64,")[1];
  }

  // 6. 检查填充字符'='的正确位置
  if (allowPadding) {
    // 填充字符只能出现在字符串末尾
    const paddingMatch = base64Str.match(/=+$/);
    if (paddingMatch) {
      const paddingLength = paddingMatch[0].length;
      // 填充字符最多只能有2个
      if (paddingLength > 2) {
        return false;
      }
      // 填充字符后面不能有其他字符
      if (base64Str.indexOf("=") !== base64Str.length - paddingLength) {
        return false;
      }
    }

    // 在严格模式下，检查填充数量是否符合Base64规则
    if (strictMode) {
      const mainLength = base64Str.replace(/=+$/, "").length;
      const modLength = mainLength % 4;

      if (modLength === 1) {
        // 填充前长度为4n+1是无效的Base64（无法从3字节映射）
        return false;
      } else if (modLength === 2) {
        // 如果余2，应该有2个填充字符
        if (paddingMatch && paddingMatch[0].length !== 2) {
          return false;
        }
      } else if (modLength === 3) {
        // 如果余3，应该有1个填充字符
        if (paddingMatch && paddingMatch[0].length !== 1) {
          return false;
        }
      }
    }
  } else if (base64Str.includes("=")) {
    // 如果不允许填充但存在填充字符
    return false;
  }

  // 7. 检查是否只包含有效字符（A-Z, a-z, 0-9, +, /, 以及可能的 =）
  const validChars = allowPadding
    ? /^[A-Za-z0-9+/]*={0,2}$/
    : /^[A-Za-z0-9+/]*$/;

  return validChars.test(base64Str);
}

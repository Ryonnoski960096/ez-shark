import type { DataItem } from "@/components/contents/model";
import type { IHeaders } from "@/stores/traffic";
// import { error } from "@tauri-apps/plugin-log";
import type { HttpRequestHeader } from "ant-design-vue/es/upload/interface";

export interface FileSizeOptions {
  precision?: number;
  base?: 1000 | 1024;
  space?: boolean;
}

/**
 * 增强版文件大小格式化
 * @param size 文件大小（以字节为单位）
 * @param options 格式化配置
 * @returns 格式化后的文件大小字符串
 */
export function formatFileSize(
  size: number,
  options: FileSizeOptions = {}
): string {
  const { precision = 2, base = 1024, space = true } = options;

  const units =
    base === 1000
      ? ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"]
      : ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];

  if (size === 0) return "0" + (space ? " " : "") + units[0];

  const i = Math.floor(Math.log(size) / Math.log(base));

  const formattedSize = parseFloat(
    (size / Math.pow(base, i)).toFixed(precision)
  );

  return `${formattedSize}${space ? " " : ""}${units[i]}`;
}

interface TimeFormatOptions {
  precision?: number;
  compact?: boolean;
}

/**
 * 毫秒转换为可读时间格式
 * @param ms 毫秒数
 * @param options 格式化配置
 * @returns 格式化后的时间字符串
 */
export function formatMilliseconds(
  ms: number,
  options: TimeFormatOptions = {}
): string {
  const { precision = 0, compact = false } = options;

  // 处理负数情况
  const absMs = Math.abs(ms);
  const sign = ms < 0 ? "-" : "";

  // 定义时间单位和转换规则
  const units = [
    { name: "d", value: 86400000 }, // 天
    { name: "h", value: 3600000 }, // 小时
    { name: "m", value: 60000 }, // 分钟
    { name: "s", value: 1000 }, // 秒
    { name: "ms", value: 1 } // 毫秒
  ];

  // 存储结果的数组
  const result: string[] = [];

  let remainingMs = absMs;

  for (const unit of units) {
    if (remainingMs >= unit.value) {
      const value = Math.floor(remainingMs / unit.value);
      const formattedValue = value.toFixed(precision);

      // 根据 compact 模式决定输出格式
      if (compact) {
        result.push(`${formattedValue}${unit.name}`);
      } else {
        result.push(`${formattedValue} ${unit.name}`);
      }

      remainingMs %= unit.value;
    }
  }

  // 如果没有任何单位，返回0
  if (result.length === 0) {
    return compact ? "0ms" : "0 ms";
  }

  // 最多显示两个单位
  return (
    sign + (result.length > 2 ? result.slice(0, 2).join(" ") : result.join(" "))
  );
}

/**
 * 将 ISO 字符串转换为时间字符串
 * @param isoString
 * @returns
 */
export const isoStringToTimeString = (isoString: string): string => {
  const date = new Date(isoString);
  return date.toTimeString().split(" ")[0];
};

export const parseUrlToHostPath = (
  url: string,
  isConnect: boolean = false
): { host: string; path: string } => {
  let host = "";
  let path = "";

  try {
    if (isConnect) {
      host = url;
      path = "";
    } else {
      // 普通 HTTP 请求的处理
      const parsedUrl = new URL(url);
      // 组合协议和主机名
      host = `${parsedUrl.protocol}//${parsedUrl.host}`;
      // 将 pathname、search 和 hash 组合成完整的路径
      path = parsedUrl.pathname + parsedUrl.search + parsedUrl.hash;
    }
  } catch {
    // error(`Error parsing URL: ${e}`);

    // 如果解析失败，尝试作为 CONNECT 地址处理
    if (url.includes(":")) {
      host = url;
      path = "";
    }
  }

  return { host, path };
};

/**
 * 将数字转换为 8 位十六进制字符串
 * @param num
 * @returns
 */
export const toHex8 = (num: number): string => {
  return num.toString(16).padStart(8, "0").toUpperCase();
};

/**
 * 将 HEX 格式的颜色转换为 RGBA 格式
 * @param hex - HEX 颜色字符串，例如 "#FF5733" 或 "#F53"
 * @param opacity - 透明度值 (0.0 - 1.0)
 * @returns 转换后的 RGBA 颜色字符串
 */
export const hexToRgba = (hex: string, opacity: number = 1): string => {
  // 移除 #
  hex = hex.replace("#", "");

  // 解析颜色值
  let r: number, g: number, b: number;

  if (hex.length === 3) {
    // #RGB 转换为 #RRGGBB
    r = parseInt(hex.charAt(0) + hex.charAt(0), 16);
    g = parseInt(hex.charAt(1) + hex.charAt(1), 16);
    b = parseInt(hex.charAt(2) + hex.charAt(2), 16);
  } else if (hex.length === 6) {
    r = parseInt(hex.substring(0, 2), 16);
    g = parseInt(hex.substring(2, 4), 16);
    b = parseInt(hex.substring(4, 6), 16);
  } else {
    throw new Error("Invalid HEX color format");
  }

  // 返回 RGBA 格式
  return `rgba(${r}, ${g}, ${b}, ${opacity})`;
};

type MarkStyleOptions = {
  backgroundColor?: string; // 背景色
  borderColor?: string; // 边框色
  borderRadius?: number; // 边角半径
  opacity?: number; // 透明度
};
/**
 * 生成 mark 样式
 * @param options
 * @returns
 */
export const generateMarkStyle = (options: MarkStyleOptions) => {
  const {
    backgroundColor = "#FFEB3B", // 默认背景色为淡黄色
    borderColor = "transparent", // 边框颜色
    borderRadius = 8, // 默认圆角
    opacity = 0.4 // 默认不透明度
  } = options;

  // 创建 RGBA 颜色字符串
  const rgbaColor = backgroundColor.startsWith("#")
    ? hexToRgba(backgroundColor, opacity)
    : backgroundColor;

  return `  
    background-color: ${rgbaColor};  
    border: 1px solid ${borderColor};  
    border-radius: ${borderRadius}px;   
    color: #333;                        
    padding: 0px 2px;                  
    font-weight: 800;            
    font-style: italic;       
    transition: background 0.3s ease;  
  `
    .replace(/\n/g, " ")
    .replace(/\s+/g, " ")
    .trim(); // 规范化样式字符串
};

/**
 * 对 HTML 字符串进行转义，防止 XSS 攻击。
 * 将特殊字符如 &, <, >, ", ' 转换为对应的 HTML 实体。
 *
 * @param unsafe - 需要转义的原始 HTML 字符串。
 * @returns 转义后的安全 HTML 字符串。
 */
export const escapeHtml = (unsafe: string) => {
  // 将字符串中的 & 替换为 &amp;
  return (
    unsafe
      .replace(/&/g, "&amp;")
      // 将字符串中的 < 替换为 &lt;
      .replace(/</g, "&lt;")
      // 将字符串中的 > 替换为 &gt;
      .replace(/>/g, "&gt;")
    // // 将字符串中的 " 替换为 &quot;
    // .replace(/"/g, "&quot;")
    // // 将字符串中的 ' 替换为 &#039;
    // .replace(/'/g, "&#039;")
  );
};

/**
 * 格式化请求头
 * 把请求头数组转为对象
 * @param obj
 * @returns
 */
export const formatHeaders = (obj: IHeaders): HttpRequestHeader => {
  try {
    return obj.items.reduce((acc: any, item: any) => {
      if (item && item.name && item.value) {
        let val = item.value;
        if (typeof val === "string") val = val.replace(/\\"/g, "");
        acc[item.name] = val;
      }
      return acc;
    }, {});
  } catch {
    return {};
  }
};

/**
 * 截断文本
 * @param text
 * @param maxLength
 * @returns
 */
export const truncateText = (text: string, maxLength = 200) => {
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength) + "...";
};

/**
 * 把cookie的字符串转为数组
 *
 * 输入：
 * a=1; b=2; c=3
 * 输出：
 * [
 *  { name: "a", value: "1" },
 *  { name: "b", value: "2" },
 *  { name: "c", value: "3" }
 * ]
 *
 * @param headerValue
 * @returns
 */
export const processCookies = (headerValue: string) => {
  const cookieList: DataItem[] = [];
  if (!headerValue) return cookieList;

  const cookies = headerValue.split(";").filter(Boolean);
  cookies.forEach((cookie) => {
    const [name, value] = cookie
      .trim()
      .split("=")
      .map((item) => item.trim());
    if (name) {
      cookieList.push({ name, value: value || "" });
    }
  });

  return cookieList;
};

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
  } catch (error) {
    console.error(`Error parsing URL: ${error}`);

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

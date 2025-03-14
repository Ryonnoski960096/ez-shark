/**
 * 解析 URL 查询参数的类型
 */
type ParsedUrlParams = Record<string, string | string[]>;

// 使用示例
export default function useUrlParams() {
  /**
   * 解析 URL 参数的方法
   * @param url 要解析的 URL 或查询字符串
   * @returns 解析后的参数对象
   */
  const parseUrlParams = (url?: string): ParsedUrlParams => {
    // 如果没有传入 URL，使用当前窗口的 URL
    const targetUrl = url || window.location.href;
    // 创建 URL 对象
    const urlObj = new URL(targetUrl, window.location.origin);

    // 获取查询参数
    const searchParams = urlObj.searchParams;

    // 存储解析结果的对象
    const result: ParsedUrlParams = {};

    // 遍历所有参数
    searchParams.forEach((value, key) => {
      // 如果已经存在该键，转换为数组
      if (result[key]) {
        // 如果已经是数组，追加
        if (Array.isArray(result[key])) {
          (result[key] as string[]).push(value);
        } else {
          // 转换为数组
          result[key] = [result[key] as string, value];
        }
      } else {
        result[key] = value;
      }
    });

    return result;
  };

  /**
   * 获取特定的 URL 参数
   * @param key 参数名
   * @param url 可选的 URL
   * @returns 参数值
   */
  const getUrlParam = <T = string>(key: string, url?: string, defaultValue?: T): T | string | undefined => {
    const params = parseUrlParams(url);
    // 获取参数值
    const value = params[key];

    // 处理数组情况
    if (Array.isArray(value)) {
      return value[0];
    }

    // 如果没有值，返回默认值
    return value || defaultValue;
  };

  /**
   * 安全地解析 JSON 类型的 URL 参数
   * @param key 参数名
   * @param url 可选的 URL
   * @returns 解析后的对象或原始值
   */
  const getJsonUrlParam = <T = any>(key: string, url?: string, defaultValue?: T): T | undefined => {
    const value = getUrlParam(key, url);

    if (!value) return defaultValue;

    try {
      return JSON.parse(decodeURIComponent(value as string));
    } catch {
      return defaultValue;
    }
  };

  const createUrlParams = (params: Record<string, any>): string => {
    // 将参数转换为查询字符串
    return Object.entries(params)
      .map(([key, value]) => {
        // 处理特殊值，比如对象需要序列化
        const formattedValue = value === null || value === undefined ? "" : typeof value === "object" ? JSON.stringify(value) : encodeURIComponent(value);

        return `${encodeURIComponent(key)}=${formattedValue}`;
      })
      .join("&");
  };

  return {
    parseUrlParams,
    getUrlParam,
    getJsonUrlParam,
    createUrlParams,
  };
}

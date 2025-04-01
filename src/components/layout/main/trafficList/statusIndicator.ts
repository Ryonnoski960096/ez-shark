export const colors = {
  info: "#909399", // 灰色 - 100系列
  success: "#67C23A", // 绿色 - 200系列
  warning: "#E6A23C", // 黄色 - 300系列
  error: "#F56C6C", // 红色 - 400系列
  fatal: "#FF0000", // 深红色 - 500系列
  default: "#000" // 默认黑色
};

export const getColorByStatus = (code: number): string => {
  const statusGroup = Math.floor(code / 100);
  switch (statusGroup) {
    case 1:
      return colors.info;
    case 2:
      return colors.success;
    case 3:
      return colors.warning;
    case 4:
      return colors.error;
    case 5:
      return colors.fatal;
    default:
      return colors.default;
  }
};

export const getStatusText = (code: number | undefined | null): string => {
  const statusTexts: Record<number, string> = {
    200: "OK",
    201: "Created",
    301: "Moved Permanently",
    302: "Found",
    304: "Not Modified",
    400: "Bad Request",
    401: "Unauthorized",
    403: "Forbidden",
    404: "Not Found",
    500: "Internal Server Error",
    502: "Bad Gateway",
    503: "Service Unavailable"
  };
  return statusTexts[code ?? 0] || "Unknown Status";
};

export const statusIndicator = (code: number | string | undefined | null) => {
  if (
    code === undefined ||
    code === null ||
    code === "null" ||
    code === "undefined"
  ) {
    return "";
  }
  if (typeof code === "string") code = parseInt(code);
  return `
  <span 
    style="
      color:${getColorByStatus(code)};
      font-weight:500;
      cursor:help;
      "
    title="${getStatusText(code)}">${code}</span>`;
};

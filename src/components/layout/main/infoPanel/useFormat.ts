import { HttpRequestHeader } from "ant-design-vue/es/upload/interface";

export function useFormat() {
  const formatHeaders = (obj: any): HttpRequestHeader => {
    try {
      return obj.items.reduce((acc: any, item: any) => {
        if (item && item.name && item.value) {
          acc[item.name] = item.value;
        }
        return acc;
      }, {});
    } catch {
      return {};
    }
  };

  const truncateText = (text: string, maxLength = 200) => {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + "...";
  };
  return { formatHeaders, truncateText };
}

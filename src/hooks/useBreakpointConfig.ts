import { useSettingStore } from "@/stores/settings";
import { createXmlStr, readXmlFile } from "@/utils/file";
import { exists, writeTextFile } from "@tauri-apps/plugin-fs";
import { error } from "@tauri-apps/plugin-log";
import { message } from "ant-design-vue";
import { computed } from "vue";

interface Request {
  header: string;
  body: string;
}

type Response = Request;

export type Method = "GET" | "POST" | "PUT" | "DELETE";

interface BreakpointConditions {
  url: string;
  method: Method | undefined;
  req_enable: boolean;
  res_enable: boolean;
  request: Partial<Request>;
  response: Partial<Response>;
}

export interface Breakpoint {
  id?: string;
  enabled: boolean;
  conditions: Partial<BreakpointConditions> & Record<string, any>;
}

export interface Breakpoints {
  breakpoints: Record<string, Breakpoint>;
  toolEnabled: boolean;
}

export default function useBreakpointConfig() {
  const settingStore = useSettingStore();

  // 获取所有断点配置
  const getAllBreakpoints = computed(() => {
    return settingStore.settings.breakpoints;
  });

  const generateTimestampKey = (): string => {
    return `breakpoint_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  };

  /**
   * 通过 key 获取单个断点配置
   * @param breakpoints 断点配置对象
   * @param key 断点的唯一标识
   * @returns 断点配置 | undefined
   */
  const getBreakpointByKey = (key: string): Breakpoint | undefined => {
    return getAllBreakpoints.value.breakpoints[key];
  };

  const importConfig = async (
    path: string,
    handler: (breakpoint: Breakpoints) => Promise<void>
  ) => {
    if (!exists(path)) throw new Error("文件不存在");

    const xmlObj = await readXmlFile(path);
    const breakpoints: Breakpoint | Breakpoint[] =
      xmlObj.breakpoints.breakpoints.breakpoint;

    const breakpointList: Breakpoint[] = Array.isArray(breakpoints)
      ? breakpoints
      : [breakpoints];

    const breakpointRecord: Record<string, Breakpoint> = {};
    breakpointList.forEach((bp) => {
      const id = bp.id || generateTimestampKey();
      breakpointRecord[id] = { ...bp, id };
    });

    const hide = message.loading("导入配置中...", 0);

    await handler({
      toolEnabled: xmlObj.breakpoints.toolEnabled,
      breakpoints: breakpointRecord
    });

    hide();
    message.success("导入配置成功");
    return breakpointRecord;
  };

  const exportConfig = async (savePath: string, breakpoints: Breakpoints) => {
    const breakpointArray = Object.values(breakpoints.breakpoints);

    const xmlObj = {
      breakpoints: {
        toolEnabled: breakpoints.toolEnabled,
        breakpoints: {
          breakpoint:
            breakpointArray.length === 1 ? breakpointArray[0] : breakpointArray
        }
      }
    };

    try {
      if (savePath) {
        const hide = message.loading("导出配置中...", 0);
        await writeTextFile(savePath, createXmlStr(xmlObj));
        hide();
        message.success("导出配置成功");
      }
    } catch (e) {
      message.error("导出配置失败");
      error("导出配置失败" + e);
    }
  };

  const createBreakpoint = (
    breakpoint: Partial<Breakpoint>
  ): [string, Breakpoint] => {
    const id = breakpoint.id || generateTimestampKey();
    return [
      id,
      {
        id,
        enabled: true,
        conditions: {},
        ...breakpoint
      }
    ];
  };

  const convertToBreakpointsRecord = (
    breakpoints: Breakpoint[]
  ): Record<string, Breakpoint> => {
    return breakpoints.reduce(
      (acc, bp) => {
        const id = bp.id || generateTimestampKey();
        acc[id] = { ...bp, id };
        return acc;
      },
      {} as Record<string, Breakpoint>
    );
  };

  return {
    getAllBreakpoints,
    getBreakpointByKey,
    importConfig,
    exportConfig,
    createBreakpoint,
    generateTimestampKey,
    convertToBreakpointsRecord
  };
}

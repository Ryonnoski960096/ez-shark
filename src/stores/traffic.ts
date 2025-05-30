import { useIpc } from "@/hooks";
import { defineStore } from "pinia";
import { ref } from "vue";
import { windowManager } from "./WindowManager";
import { BreakpointPauseEventName } from "@/enum/breakpoint";
import { emit } from "@tauri-apps/api/event";
import type { Payload } from "@/api/model";
import type { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { error } from "@tauri-apps/plugin-log";
import { useSessionStore } from "./session";
import { ezSearch } from "@/api/search";

export enum TransactionState {
  Pending = "Pending", // 初始化/等待发送
  Requesting = "Requesting", // 正在发送请求
  Responding = "Responding", // 正在接收响应
  ResponseDone = "ResponseDone", // 响应接收完成
  Completed = "Completed", // 完整完成
  Failed = "Failed", // 失败
  Aborted = "Aborted" // 中止
}

// 保留原有的 TrafficData
export interface TrafficData
  extends Record<string, number | string | null | TransactionState> {
  id: number;
  method: string;
  mime: string;
  size: number | null;
  status: number;
  time: number | null;
  uri: string;
  path: string | null;
  host: string | null;
  transaction_state: TransactionState;
  start_time: string | null;
  session_id: string;
}

// 头部项接口
interface IHeaderItem {
  name: string;
  value: string;
}

// 头部集合接口
interface IHeaders {
  items: IHeaderItem[];
  size: number;
}

// 消息体接口
interface IBodyContent {
  encode?: string;
  value?: string | null;
  size: number;
}

interface RBody {
  encode: string;
  value: string | null;
  size: number;
}

type FixedLengthArray<
  T,
  N extends number,
  A extends T[] = []
> = A["length"] extends N ? A : FixedLengthArray<T, N, [...A, T]>;

export type HexArray = FixedLengthArray<number, 16>;

export interface HexBody {
  character_view: string;
  hex: HexArray;
  offset_address: number;
}

// 完整的流量数据接口
interface ITrafficData<H = IHeaders> {
  traffic: {
    gid: number;
    uri: string;
    method: string;
    req_headers: H;
    req_body_file: string;
    req_body_hex: HexBody[];
    status: number;
    http_version: string;
    res_headers: H;
    res_body_file: string;
    res_body_size: number;
    res_body_hex: HexBody[];
    websocket_id: string | null;
    transaction_state: TransactionState;
    start_time: string;
    end_time: string;
    error: string | null;
  };
}
interface Overview extends Record<string, any> {
  url: string;
  method: string;
  status: string;
  code?: number | undefined | null;
  protocol?: string | undefined | null;
}
interface ITrafficDataDetail extends Record<string, any> {
  overview: Overview;
  req_head_json: string;
  res_head_json: string;
  req_body_hex: HexBody[];
  res_body_hex: HexBody[];
  res_body: RBody;
  req_body: RBody;
}

interface TrafficEditData<H = IHeaders> extends ITrafficData<H> {
  traffic_type: "response" | "request" | "resend";
  body: RBody | null;
}

interface TrafficWindowParams {
  url: string;
  param: {
    parentWindowId: string;
  };
}

interface WindowOptions {
  title: string;
  width: number;
}

// 定义不同类型的处理器
type NewTrafficHandler = (payload: Payload<TrafficData>) => void;
type PauseTrafficHandler = (
  payload: Payload<[string, TrafficEditData<IHeaders>]>
) => Promise<void>;
type ResendTrafficHandler = (
  payload: Payload<[string, TrafficEditData<IHeaders>]>
) => Promise<void>;

export const useTrafficStore = defineStore("traffic", () => {
  const ipc = useIpc();
  const sessionStore = useSessionStore();
  // 流量列表
  const trafficList = ref<Map<string, Map<number, TrafficData>>>(new Map());

  // 编辑状态
  const trafficEditStatusMap = ref<Map<number, boolean>>(new Map());

  // 搜索模式流量列表
  const searchMode = ref<Map<string, Map<number, TrafficData>>>(new Map());

  // 搜索模式
  const isSearchMode = ref(false);

  // 流量详情
  const trafficDetail = ref<ITrafficDataDetail | null>(null);

  // 当前选中的流量id
  const currentTrafficId = ref<Map<string, number | null>>(new Map());

  // 流量监听状态
  const isListenerMode = ref<boolean>(false);

  // 开关自动滚动
  const isAutoScroll = ref<boolean>(false);

  // 搜索流量数据的方法
  const searchTraffic = async (keyword: string) => {
    if (!sessionStore.currentSession) return;
    const hasSearchModeTraffics = searchMode.value.has(
      sessionStore.currentSession
    );
    // console.log("searchMode.value", searchMode.value);

    if (!hasSearchModeTraffics) {
      searchMode.value.set(sessionStore.currentSession, new Map());
    }

    const searchModeTraffics = searchMode.value.get(
      sessionStore.currentSession
    );
    if (!searchModeTraffics) return;
    // 如果关键词为空，退出搜索模式
    if (!keyword.trim()) {
      isSearchMode.value = false;
      return;
    }
    // 设置搜索模式为true
    // isSearchMode.value = true;

    // 创建高亮方法
    const highlightText = (text: string, keyword: string): string => {
      if (!text) return text;

      const lowercaseText = text.toLowerCase();
      const lowercaseKeyword = keyword.toLowerCase();

      // 如果文本中不包含关键词，直接返回原文本
      if (!lowercaseText.includes(lowercaseKeyword)) {
        return text;
      }

      // 使用正则表达式进行全局不区分大小写的替换
      const highlightedText = text.replace(
        new RegExp(`(${keyword})`, "gi"),
        '<span style="background-color: yellow; color: black; font-weight: bold;">$1</span>'
      );
      return highlightedText;
    };

    // console.log("searchTraffic", keyword);

    const hasTraffics = trafficList.value.has(sessionStore.currentSession);
    if (!hasTraffics) {
      trafficList.value.set(sessionStore.currentSession, new Map());
    }
    const traffics = trafficList.value.get(sessionStore.currentSession);
    if (!traffics) return;
    // 遍历trafficList进行搜索

    const ids = await ezSearch(keyword, sessionStore.currentSession);
    // console.log("ids", ids, !ids);
    if (ids.length === 0) {
      searchModeTraffics.clear();
      // console.log("searchModeTraffics", searchModeTraffics);

      return;
    }
    for (const id of ids) {
      const traffic = traffics.get(Number(id));
      if (!traffic) continue;
      // 定义高亮字段
      const highlightedTraffic = { ...traffic };
      // 如果匹配，添加到搜索结果并高亮

      // 高亮处理
      highlightedTraffic.uri = highlightText(traffic.uri, keyword);
      highlightedTraffic.method = highlightText(traffic.method, keyword);
      highlightedTraffic.mime = highlightText(traffic.mime, keyword);
      highlightedTraffic.status = Number(
        highlightText(String(traffic.status), keyword)
      );

      const hasSearchModeTraffics = searchMode.value.get(
        sessionStore.currentSession
      );
      if (!hasSearchModeTraffics) {
        searchMode.value.set(sessionStore.currentSession, new Map());
      }

      searchModeTraffics.set(id, highlightedTraffic);
    }
    // 循环searchModeTraffics，把不包含ids中的id的删掉
    const idSet = new Set(ids);

    for (const [id] of searchModeTraffics) {
      if (!idSet.has(id)) {
        searchModeTraffics.delete(id);
      }
    }
  };

  // 清除搜索
  const clearSearch = () => {
    searchMode.value.clear();
    isSearchMode.value = false;
  };

  // 被断点的流量map
  const breakpointTrafficMap = ref<Map<string, TrafficEditData>>(new Map());

  const unListenList = ref<(() => void)[]>([]);

  // 设置监听器
  const setupTrafficMonitor = async () => {
    // 抽取通用的breakpoint数据更新逻辑
    const updateBreakpointData = (key: string) => {
      breakpointTrafficMap.value.delete(key);
      emit("update:breakpointData", breakpointTrafficMap.value);
    };

    // 创建窗口的通用逻辑
    const createTrafficWindow = async (
      windowManager: any,
      setBreakpointPauseListener: (wvw: WebviewWindow) => Promise<void>
    ) => {
      const urlParams: TrafficWindowParams = {
        url: "/breakpoint/pause",
        param: {
          parentWindowId: windowManager.window.label
        }
      };

      const windowOptions: WindowOptions = {
        title: "流量编辑",
        width: 1000
      };

      const [wvw, , , isNew] = await windowManager.createWindow(
        urlParams,
        windowOptions
      );

      if (isNew) {
        await setBreakpointPauseListener(wvw);
      } else {
        emit("update:breakpointData", breakpointTrafficMap.value);
      }
    };

    // 设置断点暂停监听器
    const setBreakpointPauseListener = async (wvw: WebviewWindow) => {
      const listeners = {
        [BreakpointPauseEventName.OPEN]: () => {
          emit("update:breakpointData", breakpointTrafficMap.value);
        },
        modification: (event: { payload: string }) => {
          updateBreakpointData(event.payload);
        },
        sendTraffic: (event: { payload: string }) => {
          const key = event.payload;
          updateBreakpointData(key);
        },
        onResend: (event: { payload: string }) => {
          updateBreakpointData(event.payload);
        }
      };

      // 批量注册监听器
      Object.entries(listeners).forEach(([event, handler]) => {
        wvw.listen(event, handler);
      });
    };

    try {
      // 分别定义各种处理器
      const newTrafficHandler: NewTrafficHandler = (payload) => {
        const sessionId = payload.data.session_id;
        const trafficId = payload.data.id;
        const traffic = payload.data;
        const hasSessionTraffics = trafficList.value.has(sessionId);
        if (!hasSessionTraffics) {
          trafficList.value.set(sessionId, new Map());
        }
        const sessionTraffic = trafficList.value.get(sessionId);
        if (!sessionTraffic) return;
        sessionTraffic.set(trafficId, traffic);
        // console.log("newTrafficHandler", sessionTraffic.size);
      };

      const pauseTrafficHandler: PauseTrafficHandler = async (payload) => {
        const [key, value] = payload.data;
        breakpointTrafficMap.value.set(key, value);
        await createTrafficWindow(windowManager, setBreakpointPauseListener);
      };

      const resendTrafficHandler: ResendTrafficHandler = async (payload) => {
        const [key, value] = payload.data;
        breakpointTrafficMap.value.set(key, value);
        await createTrafficWindow(windowManager, setBreakpointPauseListener);
      };

      // 分别注册各种处理器
      const unlistenNewTraffic = await ipc.listen<Payload<TrafficData>>(
        "new-traffic",
        newTrafficHandler
      );
      unListenList.value.push(unlistenNewTraffic);

      const unlistenPauseTraffic = await ipc.listen<
        Payload<[string, TrafficEditData<IHeaders>]>
      >("pause-traffic", pauseTrafficHandler);
      unListenList.value.push(unlistenPauseTraffic);

      const unlistenResendTraffic = await ipc.listen<
        Payload<[string, TrafficEditData<IHeaders>]>
      >("resend-traffic", resendTrafficHandler);
      unListenList.value.push(unlistenResendTraffic);
    } catch (e) {
      error("Failed to setup traffic monitor:" + e);
      throw new Error(`Traffic monitor setup failed: ${e}`);
    }
  };

  const clearListen = () => {
    while (unListenList.value.length > 0) {
      unListenList.value.pop()?.();
    }
  };

  return {
    isAutoScroll,
    breakpointTrafficMap,
    isListenerMode,
    currentTrafficId,
    trafficEditStatusMap,
    trafficList,
    searchMode,
    isSearchMode,
    trafficDetail,
    unListenList,
    clearListen,
    setupTrafficMonitor,
    searchTraffic,
    clearSearch
  };
});

export type {
  ITrafficData,
  IHeaderItem,
  IHeaders,
  IBodyContent,
  ITrafficDataDetail,
  TrafficEditData,
  RBody,
  Overview
};

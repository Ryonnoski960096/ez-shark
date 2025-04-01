<template>
  <VirtualTable
    ref="virtualTableRef"
    table-id="trafficList"
    :data="tableData"
    :active="currentTrafficId"
    :columns="columns"
    :row-height="28"
    v-model:ids="ids"
    @onCellClick="onCellClick"
    @onContextMenu="oncontextmenu"
  />
</template>

<script setup lang="ts">
import { ref, useTemplateRef, watch } from "vue";
import ContextMenu from "@imengyu/vue3-context-menu";
import type { TrafficData, TransactionState } from "@/stores/traffic";
import { useTrafficStore } from "@/stores/traffic";
import { useBreakpointConfig } from "@/hooks";
import { useEventBus } from "@/hooks";
import {
  formatFileSize,
  formatMilliseconds,
  isoStringToTimeString,
  parseUrlToHostPath
} from "@/utils/format";
import VirtualTable from "@/components/VirtualTable.vue";
import { useSettingStore } from "@/stores/settings";
import type {
  Breakpoint,
  Breakpoints,
  Method
} from "@/hooks/useBreakpointConfig";
import type { Store } from "@tauri-apps/plugin-store";
import { deleteTraffic, queryTrafficDetail, resend } from "@/api/traffic";
import { copyContent } from "@/utils/tools";
import { copyApi } from "@/api/export";
import { statusIndicator } from "./statusIndicator";
import { message } from "ant-design-vue";
import { error } from "@tauri-apps/plugin-log";
import { useSessionStore } from "@/stores/session";

const settingStore = useSettingStore();
const breakpointStore = useBreakpointConfig();
const sessionStore = useSessionStore();
const trafficStore = useTrafficStore();
const virtualTableRef =
  useTemplateRef<InstanceType<typeof VirtualTable>>("virtualTableRef");

defineOptions({
  name: "TrafficList"
});

const ids = ref<Set<number>>(new Set());

const currentTrafficId = ref<number | null>(null);

watch(
  [() => sessionStore.currentSession, () => trafficStore.currentTrafficId],
  () => {
    if (!sessionStore.currentSession) {
      currentTrafficId.value = null;
      return;
    }
    const trafficId = trafficStore.currentTrafficId.get(
      sessionStore.currentSession
    );

    currentTrafficId.value = trafficId || null;
  },
  { deep: 2 }
);

// 定义列配置
const columns = [
  {
    key: "status",
    title: "Code",
    width: 7,
    minWidth: 7,
    formatter: (value: number | null) => statusIndicator(value)
  },
  {
    key: "method",
    title: "Method",
    width: 10,
    minWidth: 10
  },
  {
    key: "host",
    title: "Host",
    width: 15
  },
  {
    key: "path",
    title: "Path",
    width: 25
  },
  {
    key: "start_time",
    title: "Start",
    width: 10,
    formatter: (value: string) => isoStringToTimeString(value ?? "")
  },
  {
    key: "time",
    title: "Time",
    width: 10,
    formatter: (value: number) => formatMilliseconds(value ?? 0)
  },
  {
    key: "size",
    title: "Size",
    width: 10,
    minWidth: 10,
    formatter: (value: number | string | undefined | null) => {
      // 处理 undefined 和 null 的情况
      if (value === undefined || value === null) {
        return formatFileSize(0);
      }

      // 转换为数字，确保非 0
      value = Number(value);

      // 如果转换后是 NaN，返回 0
      return formatFileSize(isNaN(value) ? 0 : value);
    }
  },
  {
    key: "transaction_state",
    title: "Status",
    width: 15
  }
];
interface TableRowData {
  id: number;
  method: string;
  mime: string;
  size: number | null;
  status: number;
  time: number | null;
  transaction_state: TransactionState;
  start_time: string | null;
  session_id: string;
  host: string;
  path: string;
  onClick: () => void;
}

const tableData = ref<TableRowData[]>([]);

watch(
  [() => trafficStore.trafficList, () => sessionStore.currentSession],
  () => {
    const list = trafficStore.isSearchMode
      ? trafficStore.searchMode
      : trafficStore.trafficList;

    if (!sessionStore.currentSession) return;

    const traffics = list.get(sessionStore.currentSession);

    if (!traffics) {
      tableData.value = [];
      return;
    }

    tableData.value = Array.from(traffics.values()).map(({ uri, ...rest }) => {
      const { host, path } = parseUrlToHostPath(uri, rest.method === "CONNECT");
      return {
        ...rest,
        host: host,
        path: decodeURIComponent(path),
        onClick: () => handleRowClick(rest.id)
      };
    });
  },
  {
    immediate: true,
    deep: 3
  }
);

/**
 * 添加断点
 * @param traffic
 */
const addBreakpoint = async (traffic: TrafficData) => {
  const breakpoint: Breakpoint = {
    enabled: true,
    conditions: {
      url: traffic.host + "" + traffic.path,
      method: traffic.method as Method,
      req_enable: true,
      res_enable: true,
      request: {},
      response: {}
    }
  };
  if ((await settingStore.store) === null) return;

  const breakpointList: Breakpoints | null | undefined = await (
    (await settingStore.store) as Store
  ).get("breakpoints");
  let breakpoints = breakpointList ?? {
    toolEnabled: false,
    breakpoints: {}
  };
  if (!breakpoints) {
    breakpoints = {
      toolEnabled: false,
      breakpoints: {}
    };
  }
  const [key, b] = breakpointStore.createBreakpoint(breakpoint);
  breakpoints.breakpoints[key] = b;

  settingStore.set("breakpoints", breakpoints);
};

/**
 * 验证流量是否已添加断点
 * @param traffic 流量数据
 * @returns 是否已添加断点
 */
const isTrafficBreakpointAdded = async (
  traffic: TrafficData
): Promise<boolean> => {
  // 如果设置存储为空，直接返回 false
  if ((await settingStore.store) === null) return false;

  // 获取断点列表
  const breakpointList: Breakpoints | null | undefined = await (
    (await settingStore.store) as Store
  ).get("breakpoints");

  // 如果没有断点列表，返回 false
  if (!breakpointList || !breakpointList.breakpoints) return false;

  // 遍历现有断点
  return Object.values(breakpointList.breakpoints).some(
    (breakpoint) =>
      breakpoint.conditions.url === (traffic.host ?? "") + traffic.path &&
      breakpoint.conditions.method === traffic.method
  );
};

/**
 * 删除匹配的断点
 * @param traffic 流量数据
 * @returns 是否成功删除断点
 */
const removeBreakpoint = async (traffic: TrafficData): Promise<boolean> => {
  // 如果设置存储为空，直接返回 false
  if ((await settingStore.store) === null) return false;

  // 获取断点列表
  const breakpointList: Breakpoints | null | undefined = await (
    (await settingStore.store) as Store
  ).get("breakpoints");

  // 如果没有断点列表，返回 false
  if (!breakpointList || !breakpointList.breakpoints) return false;

  // 找到匹配的断点 key
  const matchedBreakpointKeys = Object.entries(breakpointList.breakpoints)
    .filter(
      ([, breakpoint]) =>
        breakpoint.conditions.url === (traffic.host ?? "") + traffic.path &&
        breakpoint.conditions.method === traffic.method
    )
    .map(([key]) => key);

  // 如果没有匹配的断点，返回 false
  if (matchedBreakpointKeys.length === 0) return false;

  // 删除匹配的断点
  matchedBreakpointKeys.forEach((key) => {
    delete breakpointList.breakpoints[key];
  });

  // 更新断点列表
  await settingStore.set("breakpoints", breakpointList);

  return true;
};

async function oncontextmenu(e: MouseEvent, traffic: TrafficData) {
  e.preventDefault();
  if (ids.value.size !== 0) {
    ContextMenu.showContextMenu({
      x: e.x,
      y: e.y,
      items: [
        {
          label: "Delete Selected",
          onClick: async () => {
            try {
              if (!sessionStore.currentSession) return;

              const currentSessionTraffics = trafficStore.trafficList.get(
                sessionStore.currentSession
              );
              if (!currentSessionTraffics) return;
              await deleteTraffic([...ids.value]);

              ids.value.forEach((id) => {
                currentSessionTraffics.delete(id);
              });
              ids.value.clear();
              trafficStore.currentTrafficId.set(
                sessionStore.currentSession,
                currentSessionTraffics.keys().next().value ?? 0
              );
              message.success("删除成功");
            } catch (e) {
              error("删除失败：" + e);
              message.error("删除失败");
            }
          }
        }
      ]
    });
    return;
  }
  if (!sessionStore.currentSession) return;

  trafficStore.currentTrafficId.set(sessionStore.currentSession, traffic.id);

  const id = traffic.id;
  const isAdded = await isTrafficBreakpointAdded(traffic);

  ContextMenu.showContextMenu({
    x: e.x,
    y: e.y,
    items: [
      {
        label: `${isAdded ? "√ " : ""}Breakpoint`,
        onClick: () =>
          isAdded ? removeBreakpoint(traffic) : addBreakpoint(traffic)
      },
      {
        label: "Resend",
        onClick: async () => {
          await resend(id);
        }
      },
      {
        label: "Copy URL",
        onClick: () => {
          const text = (traffic.host ?? "") + traffic.path;
          copyContent(text);
        }
      },
      {
        label: "Delete",
        onClick: async () => {
          try {
            if (!sessionStore.currentSession) return;

            const currentSessionTraffics = trafficStore.trafficList.get(
              sessionStore.currentSession
            );
            if (!currentSessionTraffics) return;
            await deleteTraffic([id]);
            currentSessionTraffics.delete(id);
            trafficStore.currentTrafficId.set(
              sessionStore.currentSession,
              currentSessionTraffics.keys().next().value ?? 0
            );
          } catch (e) {
            error("删除失败：" + e);
            message.error("删除失败");
          }
        }
      },
      {
        label: "Copy",
        children: [
          {
            label: "Request Body",
            onClick: async () => {
              copyContent(await copyApi(id, "req-body"));
            }
          },
          {
            label: "Response Body",
            onClick: async () => {
              copyContent(await copyApi(id, "res-body"));
            }
          },
          {
            label: "Markdown",
            onClick: async () => {
              copyContent(await copyApi(id, "markdown"));
            }
          },
          {
            label: "Har",
            onClick: async () => {
              copyContent(await copyApi(id, "har"));
            }
          },
          {
            label: "Curl",
            onClick: async () => {
              copyContent(await copyApi(id, "curl"));
            }
          }
        ]
      }
    ]
  });
}

function handleRowClick(id: number) {
  if (!sessionStore.currentSession) return;

  trafficStore.currentTrafficId.set(sessionStore.currentSession, id);
}
const onCellClick = (item: any) => {
  handleRowClick(item.id);
};

const eventBus = useEventBus();

// 监听活跃行变化
watch(
  [() => trafficStore.currentTrafficId, () => sessionStore.currentSession],
  async () => {
    if (!sessionStore.currentSession) return;
    const id = trafficStore.currentTrafficId.get(sessionStore.currentSession);
    if (!id) {
      eventBus.emit("change:trafficDetail", null);
      return;
    }

    if (id && id !== 0) {
      // if (!sessionStore.currentSession) return;

      const currentSessionTraffics = trafficStore.trafficList.get(
        sessionStore.currentSession
      );
      if (!currentSessionTraffics) return;

      const traffic = currentSessionTraffics.get(id);
      if (!traffic) return;
      const res = await queryTrafficDetail(traffic.id);
      trafficStore.trafficDetail = res;
      eventBus.emit("change:trafficDetail", trafficStore.trafficDetail);
    }
  },
  {
    deep: true
  }
);

defineExpose({
  virtualTableRef
});
</script>

<style scoped>
.container {
  height: 100%;
  width: 100%;
}
</style>

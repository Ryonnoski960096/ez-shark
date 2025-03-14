<template>
  <VirtualTable
    table-id="trafficList"
    :data="tableData"
    :active="trafficStore.currentTrafficId"
    :columns="columns"
    :row-height="28"
    @onCellClick="onCellClick"
    @onContextMenu="oncontextmenu"
  />
</template>

<script setup lang="ts">
import { computed, watch } from "vue";
import ContextMenu from "@imengyu/vue3-context-menu";
import { TrafficData, useTrafficStore } from "@/stores/traffic";
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
import { Breakpoint, Breakpoints, Method } from "@/hooks/useBreakpointConfig";
import { Store } from "@tauri-apps/plugin-store";
import { updateBreakpoint } from "@/api/breakpoint";
import { queryTrafficDetail, resend } from "@/api/traffic";
import { copyContent } from "@/utils/tools";
import { copyApi } from "@/api/export";
import { statusIndicator } from "./statusIndicator";

defineOptions({
  name: "TrafficList"
});

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
    formatter: (value: number) => formatFileSize(value ?? 0)
  },
  {
    key: "transaction_state",
    title: "Status",
    width: 15
  }
];

// 计算表格数据
const tableData = computed(() => {
  const list = trafficStore.isSearchMode
    ? trafficStore.searchMode
    : trafficStore.trafficList;

  return Array.from(list.values()).map(({ uri, ...rest }) => {
    const { host, path } = parseUrlToHostPath(uri, rest.method === "CONNECT");

    return {
      ...rest,
      host: host,
      path: path,
      onClick: () => handleRowClick(rest.id)
    };
  });
});

const settingStore = useSettingStore();
const breakpointStore = useBreakpointConfig();

/**
 * 添加断点
 * @param traffic
 */
const addBreakpoint = async (traffic: TrafficData) => {
  const breakpoint: Breakpoint = {
    enabled: true,
    conditions: {
      url: traffic.uri,
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

  const breakpointLists = [];
  for (const key in breakpoints.breakpoints) {
    breakpointLists.push(breakpoints.breakpoints[key]);
  }

  await updateBreakpoint(breakpointLists);
};

function oncontextmenu(e: MouseEvent, traffic: TrafficData) {
  e.preventDefault();
  trafficStore.currentTrafficId = traffic.id;

  const id = traffic.id;
  ContextMenu.showContextMenu({
    x: e.x,
    y: e.y,
    items: [
      {
        label: "Breakpoint",
        onClick: () => addBreakpoint(traffic)
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
          let text = (traffic.host ?? "") + traffic.path;
          copyContent(text);
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

const onCellClick = (item: any) => {
  trafficStore.currentTrafficId = item.id;
};
function handleRowClick(id: number) {
  trafficStore.currentTrafficId = id;
}

const trafficStore = useTrafficStore();

const eventBus = useEventBus();

// 监听活跃行变化
watch(
  () => trafficStore.currentTrafficId,
  async (newValue) => {
    if (!newValue) {
      eventBus.emit("change:trafficDetail", null);
      return;
    }

    if (newValue && newValue !== 0) {
      const traffic = trafficStore.trafficList.get(newValue);
      if (!traffic) return;
      const res = await queryTrafficDetail(traffic.id);

      trafficStore.trafficDetail = res;
      eventBus.emit("change:trafficDetail", trafficStore.trafficDetail);
    }
  }
);
</script>

<style scoped>
.container {
  height: 100%;
  width: 100%;
}
</style>

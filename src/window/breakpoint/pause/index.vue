<template>
  <div class="pause">
    <Panel ref="leftPanel" class="list w-20vw">
      <List
        v-model="currentKey"
        :items="trafficStore.breakpointTrafficMap"
        @select="handleSelect"
      />
    </Panel>
    <Splitter ref="splitterRef" />
    <Panel ref="rightPanel" class="edit f-col-between-center w h-100% p-2px">
      <!-- <div class="f-col-between-center h-100% w"> style="height: calc(100% - 32px)" -->
      <div class="w f-col-start-center" style="height: calc(100vh - 84px)">
        <div class="sticky-header w">
          <Tabs :tabs="computedGetTabs" v-model="activeTab" />
        </div>
        <InfoContent v-show="activeTab === '0'" :overview="overview" />
        <div v-if="activeTab === '1'" class="w h-100%">
          <KeepAlive>
            <URLEditor
              v-if="editTab === 'URL'"
              v-model:urlEditData="infoParams.url.urlEditData"
              v-model:dataSource="infoParams.url.dataSource"
            />
            <JsonEditorVue
              v-else-if="editTab === 'Header'"
              v-model="infoParams.header"
              v-bind="JsonEditorVueProps"
            />
            <EditTable
              v-else-if="editTab === 'Cookie'"
              v-model:dataSource="infoParams.cookie"
            />
            <Text
              v-else-if="editTab === 'Text'"
              v-model:content="infoParams.text"
            />
            <JsonEditorVue
              v-else-if="editTab === 'JSON Text'"
              v-model="infoParams.text"
              v-bind="JsonEditorVueProps"
            />
          </KeepAlive>
        </div>
      </div>
      <div class="operate">
        <div class="w f-l b-#F0F0F0">
          <Segmented
            class="w"
            v-model:value="editTab"
            v-if="activeTab === '1'"
            :options="editTabs"
            size="small"
          />
        </div>
        <Space :size="10">
          <Button @click="onCancel" size="small">Cancel</Button>
          <Button @click="onAbort" size="small">Abort</Button>
          <Button @click="onExecute" size="small">Execute</Button>
        </Space>
      </div>
      <!-- </div> -->
    </Panel>
  </div>
</template>

<script setup lang="ts">
import Panel from "@/components/Panel.vue";
import List from "./list/index.vue";
import Splitter from "@/components/Splitter.vue";
import Tabs from "@/components/tabs/index.vue";
import { Button, Segmented, Space } from "ant-design-vue";
import InfoContent from "@/components/layout/main/infoPanel/index.vue";

import { ref, onMounted, computed, watch } from "vue";
import type { Tab } from "@/components/tabs/model";
import type { IHeaders, Overview, TrafficEditData } from "@/stores/traffic";
import { useTrafficStore } from "@/stores/traffic";
import { listen } from "@tauri-apps/api/event";
import type {
  Params,
  TabType,
  trafficModificationAPIParams
} from "../../../components/contents/model";
import { requiredTabs } from "../../../components/contents/model";

import { BreakpointPauseEventName } from "@/enum/breakpoint";
import { windowInit, windowManager } from "@/stores/WindowManager";
import { onResend, trafficContinue, trafficModification } from "@/api/traffic";
import { isSuccess } from "@/api";
import JsonEditorVue from "json-editor-vue";
import EditTable from "@/components/EditTable.vue";
import URLEditor from "@/components/contents/url.vue";
import Text from "@/components/contents/text.vue";
import { deepClone } from "@/utils/tools";
import type { HttpRequestHeader } from "ant-design-vue/es/upload/interface";
import { formatHeaders, processCookies } from "@/utils/format";
import { error } from "@tauri-apps/plugin-log";

windowInit();

enum Mode {
  text = "text",
  tree = "tree",
  table = "table"
}
const JsonEditorVueProps = {
  mode: Mode.text,
  mainMenuBar: false,
  statusBar: false,
  navigationBar: false,
  askToFormat: false,
  flattenColumns: true,
  style: {
    height: "100%"
  }
};
const trafficStore = useTrafficStore();
const activeTab = ref<string>("0");

const overview = ref<Partial<Overview>>({});

const defaultParams: Omit<Params, "json" | "hex" | "imgBase64"> = {
  url: {
    dataSource: [],
    urlEditData: {
      method: "",
      url: "",
      params: {},
      httpVersion: ""
    }
  },
  header: "",
  text: "",
  cookie: []
};
const infoParams = ref<Omit<Omit<Params, "hex">, "json">>(
  deepClone(defaultParams)
);
const trafficType = ref();

// 重置参数
const reSetParams = () => {
  overview.value = {};
  infoParams.value = deepClone(defaultParams);
  trafficType.value = undefined;
};

// 外部控制选中的索引
const currentKey = ref<string | undefined>();

// Panel 和 Splitter 的引用
const leftPanel = ref<InstanceType<typeof Panel> | null>(null);
const rightPanel = ref<InstanceType<typeof Panel> | null>(null);
const splitterRef = ref<InstanceType<typeof Splitter> | null>(null);
const editTab = ref<TabType>("URL");

const editTabs = ref([`URL`, ...requiredTabs]);

// 操作
const onCancel = () => {};
const onAbort = () => {};

// 处理url dataSource数据
const handleUrl = (params: Record<string, string>) => {
  try {
    // 构建 dataSource
    return Object.entries(params).map(([name, value]) => ({
      name,
      value: value as string
    }));
  } catch {
    return [];
  }
};

// 头部数据处理
const headerDataSourceHandel = (header: HttpRequestHeader) => {
  try {
    return JSON.stringify(header, null, 2);
  } catch (error) {
    console.warn("处理头部数据时出错:", error);
    return "";
  }
};

// 处理选中事件
const handleSelect = ({ key }: { key: string }) => {
  reSetParams();
  const breakpointTraffic = trafficStore.breakpointTrafficMap.get(key) ?? null;
  const traffic = breakpointTraffic?.traffic;
  if (!breakpointTraffic || !traffic) return;

  currentKey.value = key;
  trafficType.value = breakpointTraffic.traffic_type;

  const url = traffic.uri;
  const method = traffic.method;
  const protocol = traffic.http_version;

  // 处理overview数据
  overview.value = {
    url,
    method,
    protocol,
    status: traffic.transaction_state,
    code: traffic.status
  };
  // 处理url
  const urlObj = new URL(url);
  const params = Object.fromEntries(urlObj.searchParams);
  infoParams.value.url.urlEditData = {
    method,
    params,
    url: `${urlObj.origin}${urlObj.pathname}`,
    httpVersion: protocol
  };
  infoParams.value.url.dataSource = handleUrl(params);

  // 处理header
  try {
    const h = formatHeaders(traffic.req_headers);
    infoParams.value.header = headerDataSourceHandel(h);

    // 处理 Cookie
    const cookieHeader = h.cookie;
    if (cookieHeader) {
      infoParams.value.cookie = processCookies(cookieHeader);
    }
  } catch {}

  // 处理请求体
  const bodyValue = breakpointTraffic.body?.value;
  if (bodyValue) {
    try {
      const json = JSON.parse(bodyValue);
      const formatBody = JSON.stringify(json, null, 2);
      infoParams.value.text = formatBody;
    } catch {
      infoParams.value.text = bodyValue;
    }
  }
};

// 在挂载后设置元素
onMounted(async () => {
  if (splitterRef.value && leftPanel.value && rightPanel.value) {
    const leftElement = leftPanel.value.$el as HTMLElement;
    const rightElement = rightPanel.value.$el as HTMLElement;
    splitterRef.value.setElements(leftElement, rightElement);
  }
  await windowManager.window.emit(BreakpointPauseEventName.OPEN, true);
});

const getTabs = (): Tab[] => {
  const tab = [{ id: "0", label: "Overview" }];
  if (trafficType.value === "request") {
    tab.push({ id: "1", label: "Edit Request" });
  } else if (trafficType.value === "response") {
    tab.push({ id: "1", label: "Edit Response" });
  } else if (trafficType.value === "resend") {
    tab.push({ id: "1", label: "Send Traffic" });
  }
  return tab;
};

const computedGetTabs = computed(() => getTabs());

listen("update:breakpointData", (e) => {
  const data = e.payload as Record<string, TrafficEditData<IHeaders>>;
  const dataKeySet = new Set(Object.keys(data));
  const trafficKeySet = new Set(trafficStore.breakpointTrafficMap.keys());

  // 新增的键（在 data 中有，但在 trafficMap 中没有）
  const addedKeys = [...dataKeySet].filter((x) => !trafficKeySet.has(x));

  // 删除的键（在 trafficMap 中有，但在 data 中没有）
  const removedKeys = [...trafficKeySet].filter((x) => !dataKeySet.has(x));

  // 处理新增的键
  addedKeys.forEach((key) => {
    trafficStore.breakpointTrafficMap.set(key, data[key]);
  });

  // 处理删除的键
  removedKeys.forEach((key) => {
    trafficStore.breakpointTrafficMap.delete(key);
  });
});

const getFirstKey = (map: Map<any, any>) => {
  if (map.size === 0) return undefined;
  return map.keys().next().value;
};

// 执行
const onExecute = async () => {
  if (!currentKey.value) return;
  const url = infoParams.value.url;
  const params = url.dataSource;

  // 构建查询字符串
  const queryString = params
    .filter((param) => param.name) // 过滤掉没有 name 的参数
    .map(
      (param) =>
        `${encodeURIComponent(param.name)}=${encodeURIComponent(param.value || "")}`
    )
    .join("&");

  // 添加到 URL 中
  const data: trafficModificationAPIParams = {
    id: currentKey.value,
    url: `${url.urlEditData.url}${queryString ? "?" + queryString : ""}`,
    method: url.urlEditData.method
  };

  try {
    // const headerParamsObj = ;
    data.modified_headers = JSON.parse(infoParams.value.header ?? "");
  } catch {
    error("headerParams 解析失败，但不影响代码执行");
  }

  const modified_body = infoParams.value.text;

  if (modified_body) {
    data.modified_body = modified_body;
  }

  if (!trafficType.value) return;
  let res;

  switch (trafficType.value) {
    case "request":
    case "response":
      res = await trafficModification(trafficType.value, data);
      break;
    case "resend":
      res = await onResend(data);
  }
  const trafficStore = useTrafficStore();
  if (isSuccess(res)) {
    trafficStore.breakpointTrafficMap.delete(data.id);
    reSetParams();
    activeTab.value = "0";

    currentKey.value = getFirstKey(trafficStore.breakpointTrafficMap);
    if (currentKey.value) handleSelect({ key: currentKey.value });

    if (trafficType.value === "resend") {
      windowManager.window.emit("sendTraffic", data.id);
    } else {
      windowManager.window.emit("modification", data.id);
      trafficContinue(data.id);
    }
  }
};

// 控制是否需要触发更新
let isUpdatingFromCookie = false;
let isUpdatingFromHeader = false;

// 监听 Cookie 变化，同步到 Header
watch(
  () => infoParams.value.cookie,
  (newValue) => {
    // 如果是从 header 更新触发的，则不需要再更新回 header
    if (isUpdatingFromHeader) {
      isUpdatingFromHeader = false;
      return;
    }

    isUpdatingFromCookie = true;
    try {
      // 构建 Cookie 字符串
      const cookieStr = newValue.map((v) => `${v.name}=${v.value}`).join(";");

      // 更新 Header 中的 Cookie
      const headerObj = JSON.parse(infoParams.value.header ?? "");
      headerObj.cookie = cookieStr.length !== 0 ? cookieStr : undefined;
      infoParams.value.header = JSON.stringify(headerObj, null, 2).replace(
        /\\"/g,
        ""
      );
    } catch {
      error("处理 Cookie 时出错，但不影响代码执行");
    }
    isUpdatingFromCookie = false;
  },
  { deep: 2 }
);

// 监听 header 变化并更新到 cookie
watch(
  () => infoParams.value.header,
  (newHeader) => {
    // 如果是从 cookie 更新触发的，则不需要再更新回 cookie
    if (isUpdatingFromCookie) {
      isUpdatingFromCookie = false;
      return;
    }

    isUpdatingFromHeader = true;
    try {
      const headerCookie = JSON.parse(newHeader).cookie;
      if (headerCookie) {
        // 清空现有的 cookie
        infoParams.value.cookie = [];

        const cookieArr = headerCookie.split(";");
        for (const cookie of cookieArr) {
          const [name, value] = cookie.split("=");
          infoParams.value.cookie.push({
            name: name.trim(),
            value: value.trim()
          });
        }
      }
    } catch {
    } finally {
      isUpdatingFromHeader = false;
    }
  }
);

watch(
  () => trafficStore.breakpointTrafficMap.size,
  () => {
    currentKey.value = getFirstKey(trafficStore.breakpointTrafficMap);
    if (currentKey.value) {
      handleSelect({ key: currentKey.value });
    }
  }
);
</script>

<style scoped>
.pause {
  height: calc(100vh - 32px);
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  justify-content: flex-start;
}

/* 防止初始状态下Panel过于紧凑 */
.list {
  min-width: 50px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  height: 100%;
}
.edit {
  white-space: nowrap;
  overflow-y: auto;
  text-align: left;
}
.sticky-header {
  overflow: hidden;
  padding: 2px;
  position: sticky;
  top: 0;
  background-color: white; /* 添加背景色，防止内容穿透 */
  z-index: 10; /* 确保在其他内容之上 */
  padding-bottom: 5px;
  min-height: 32px;
}

.operate {
  width: 100%;
  background-color: #f5f5f5;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}
</style>

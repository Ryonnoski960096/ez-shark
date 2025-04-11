<template>
  <div class="split-container">
    <div ref="topPanel" class="panel panel--top">
      <div class="top-panel-container">
        <div class="traffic-list-wrapper">
          <TrafficList ref="trafficListRef" class="trafficList" />
        </div>
        <div class="search-container">
          <InputSearch
            size="small"
            v-model:value="value"
            placeholder="Search"
            style="width: 100%"
          />
        </div>
      </div>
    </div>
    <Splitter ref="splitter1" direction="horizontal" :min-size="100" />
    <div style="height: 300px" class="bottom-container">
      <Tabs :tabs="tabs" v-model="activeTab" />
      <div ref="middlePanel" style="height: 100px" class="panel panel--middle">
        <Panel class="w h-100%" v-show="activeTab === '0'">
          <InfoContent :overview="infoParams.overview" />
        </Panel>
        <Panel class="pos-relative w h-100% p-2px" v-show="activeTab === '1'">
          <div
            class="f-col-between-center w edit-table"
            style="
              overflow-x: hidden;
              border: 1px solid #00000030;
              border-radius: 3px;
            "
          >
            <KeepAlive>
              <Json
                class="w p-2"
                v-if="requestTab === 'Header'"
                ref="reqHeaderRef"
                :readOnly="true"
                v-model:value="infoParams.req.header"
                highlightNodeId="request-header-json-mask"
              />
              <EditTable
                v-else-if="requestTab === 'Cookie'"
                :readOnly="true"
                v-model:dataSource="infoParams.req.cookie"
              />
              <Text
                v-else-if="requestTab === 'Text'"
                ref="reqTextRef"
                :readOnly="true"
                v-model:content="infoParams.req.text"
              />
              <JsonEditorVue
                class="w"
                v-else-if="requestTab === 'JSON Text' && infoParams.req.json"
                :readOnly="true"
                v-model="infoParams.req.json"
                v-bind="JsonEditorVueProps"
              />
              <Hex
                v-else-if="requestTab === 'Hex'"
                :hexBody="infoParams.req.hex"
              />
            </KeepAlive>
          </div>

          <div class="operate">
            <Segmented
              class="w"
              v-model:value="requestTab"
              :options="requestTabs"
              size="small"
            />
          </div>
        </Panel>
      </div>
      <Splitter
        v-show="activeTab !== '0'"
        ref="splitter2"
        direction="horizontal"
        :min-size="100"
      />
      <div
        v-show="activeTab !== '0'"
        ref="bottomPanel"
        style="height: 200px"
        class="panel panel--bottom"
      >
        <Panel class="pos-relative w h-100% p-2px">
          <div
            class="f-col-between-start w edit-table"
            style="
              overflow-x: hidden;
              border: 1px solid #00000030;
              border-radius: 3px;
            "
          >
            <KeepAlive>
              <Json
                highlightNodeId="response-header-json-mask"
                class="w"
                v-if="responseTab === 'Header'"
                ref="resHeaderRef"
                :readOnly="true"
                v-model:value="infoParams.res.header"
              />
              <EditTable
                v-else-if="responseTab === 'Cookie'"
                :readOnly="true"
                v-model:dataSource="infoParams.res.cookie"
              />
              <Text
                v-else-if="responseTab === 'Text'"
                ref="resTextRef"
                :readOnly="true"
                v-model:content="infoParams.res.text"
              />
              <JsonEditorVue
                class="w"
                v-else-if="responseTab === 'JSON Text' && infoParams.res.json"
                :readOnly="true"
                v-model="infoParams.res.json"
                v-bind="JsonEditorVueProps"
              />
              <Hex
                v-else-if="responseTab === 'Hex'"
                :hexBody="infoParams.res.hex"
              />
              <Image
                v-else-if="responseTab === 'Image'"
                :base64="infoParams.res.imgBase64"
              />
            </KeepAlive>
          </div>
          <div class="operate">
            <Segmented
              class="w"
              v-model:value="responseTab"
              :options="responseTabs"
              size="small"
            />
          </div>
        </Panel>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import TrafficList from "./trafficList/index.vue";
import InfoContent from "@/components/layout/main/infoPanel/index.vue";
import Tabs from "@/components/tabs/index.vue";
import Splitter from "@/components/Splitter.vue";
import type { Ref } from "vue";
import { nextTick, onMounted, ref, useTemplateRef, watch } from "vue";
import { useEventBus } from "@/hooks";
import type { Overview } from "@/stores/traffic";
import { useTrafficStore } from "@/stores/traffic";
import type { Tab } from "@/components/tabs/model";
import { InputSearch, Segmented } from "ant-design-vue";
import {
  requiredTabs as onURLRequiredTabs,
  type Params,
  type TabType
} from "@/components/contents/model";
import Hex from "@/components/contents/hex.vue";
import Json from "@/components/contents/json.vue";
import EditTable from "@/components/EditTable.vue";
import Text from "@/components/contents/text.vue";
import Panel from "@/components/Panel.vue";
import { useIntervalFn } from "@vueuse/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { ActiveTraffic } from "@/utils/eventBus";
import { deepClone, isValidBase64, waitForCondition } from "@/utils/tools";
import JsonEditorVue from "json-editor-vue";
import { processCookies } from "@/utils/format";
import { useSessionStore } from "@/stores/session";
import Image from "@/components/contents/image.vue";

const value = ref("");
const trafficStore = useTrafficStore();
const sessionStore = useSessionStore();

const { pause, resume, isActive } = useIntervalFn(
  () => {
    trafficStore.searchTraffic(value.value);
  },
  500,
  { immediate: false }
);

watch(value, (newValue) => {
  if (newValue.length === 0) {
    trafficStore.clearSearch();
    if (isActive.value) pause();
  } else {
    if (!isActive.value) resume();
  }
});

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
defineOptions({
  // eslint-disable-next-line vue/no-reserved-component-names
  name: "Main"
});

const activeTab = ref<string>("0");
const requestTab = ref<TabType>("Header");
const responseTab = ref<TabType>("Header");

const defaultInfoParams: Omit<Params, "url" | "imgBase64"> = {
  header: "",
  cookie: [],
  text: "",
  json: undefined,
  hex: []
};

// 构建参数
const infoParams = ref<{
  overview: Partial<Overview>;
  req: Omit<Params, "url" | "imgBase64">;
  res: Omit<Params, "url">;
}>({
  overview: {},
  req: deepClone(defaultInfoParams),
  res: deepClone({
    ...defaultInfoParams,
    imgBase64: ""
  })
});

// 重置参数
const reSetInfoParams = () => {
  infoParams.value = {
    overview: {},
    req: deepClone(defaultInfoParams),
    res: deepClone(defaultInfoParams)
  };
};

const topPanel = ref<HTMLElement | null>(null);
const middlePanel = ref<HTMLElement | null>(null);
const bottomPanel = ref<HTMLElement | null>(null);
const splitter1 = ref<InstanceType<typeof Splitter> | null>(null);
const splitter2 = ref<InstanceType<typeof Splitter> | null>(null);

// 分割线init
onMounted(() => {
  if (
    topPanel.value &&
    middlePanel.value &&
    middlePanel.value.parentElement &&
    bottomPanel.value &&
    splitter1.value &&
    splitter2.value
  ) {
    splitter1.value.setElements(
      topPanel.value,
      middlePanel.value.parentElement
    );
    splitter2.value.setElements(middlePanel.value, bottomPanel.value);
  }
});

const cookieInit = (jsonStr: string) => {
  try {
    const json = JSON.parse(jsonStr);
    const cookies = json["cookie"];
    return processCookies(cookies);
  } catch {}
};

const isUpdated = ref(false);

function formatQueryString(str: string): string {
  // 如果不是有效的查询字符串，返回原始字符串
  if (!/^(&)?([^=&]+)=(.*)(&([^=&]+)=(.*))*$/.test(str)) {
    return str;
  }

  // 移除可能的开头 &
  str = str.replace(/^&/, "");

  // 分割查询参数
  const params = str.split("&");

  // 按照 key 的字母顺序排序
  const sortedParams = params.sort((a, b) => {
    const keyA = a.split("=")[0];
    const keyB = b.split("=")[0];
    return keyA.localeCompare(keyB);
  });

  // 格式化，每个参数单独一行
  return sortedParams
    .map((param) => {
      const [key, value] = param.split("=");
      return `${key}=${value}`;
    })
    .join("\n");
}

const { on } = useEventBus();
on("change:trafficDetail", (data) => {
  isUpdated.value = false;
  // 重置
  reSetInfoParams();
  if (!data) return;
  if (data?.overview) {
    infoParams.value.overview = data.overview;
  }
  // request
  if (data.req_head_json) {
    infoParams.value.req.header = data.req_head_json;
    const reqCookieDataItem = cookieInit(data.req_head_json);
    if (reqCookieDataItem) {
      infoParams.value.req.cookie = reqCookieDataItem;
    }
  }
  if (data.req_body_hex) {
    infoParams.value.req.hex = data.req_body_hex;
  }
  if (data.req_body) {
    infoParams.value.req.text = formatQueryString(
      deepClone(data.req_body.value) ?? ""
    );
    try {
      data.req_body.value && JSON.parse(data.req_body.value);
      infoParams.value.req.json = deepClone(data.req_body.value);
    } catch {}
  }
  // response
  if (data.res_head_json) {
    infoParams.value.res.header = data.res_head_json;
    const resCookieDataItem = cookieInit(data.res_head_json);
    if (resCookieDataItem) {
      infoParams.value.res.cookie = resCookieDataItem;
    }
  }
  if (data.res_body_hex) {
    infoParams.value.res.hex = data.res_body_hex;
  }
  if (data.res_body && data.res_body.value) {
    infoParams.value.res.text = formatQueryString(
      deepClone(data.res_body.value) ?? ""
    );
    try {
      JSON.parse(data.res_body.value);
      infoParams.value.res.json = data.res_body.value ?? "";
    } catch {}
    if (isValidBase64(data.res_body.value)) {
      infoParams.value.res.imgBase64 = data.res_body.value;
    }
  }
  // 更新tab
  updateParamTabs(infoParams.value.req, requestTabs, requestTab);
  updateParamTabs(infoParams.value.res, responseTabs, responseTab);
  isUpdated.value = true;
});

const requestTabs = ref<TabType[]>(deepClone([...onURLRequiredTabs, "Hex"]));

const responseTabs = ref<TabType[]>(deepClone([...onURLRequiredTabs, "Hex"]));

const updateParamTabs = (
  newParams: any,
  tabsRef: Ref<string[]>,
  tab: Ref<TabType>
) => {
  const paramTypes = [
    { key: "header", tabName: "Header" },
    { key: "cookie", tabName: "Cookie" },
    { key: "text", tabName: "Text" },
    { key: "json", tabName: "JSON Text" },
    { key: "hex", tabName: "Hex" },
    { key: "imgBase64", tabName: "Image" }
  ];

  paramTypes.forEach((param) => {
    const paramValue = newParams?.[param.key];

    // 统一的检查逻辑
    const isValid = paramValue && paramValue.length > 0;

    // 使用 Set 来确保唯一性
    const currentTabs = new Set(tabsRef.value);

    if (isValid) {
      // 只有在不存在时才添加
      if (!currentTabs.has(param.tabName)) {
        tabsRef.value.push(param.tabName);
      }
    } else {
      // 如果无效，移除对应的tab
      const index = tabsRef.value.indexOf(param.tabName);
      if (index > -1) {
        tabsRef.value.splice(index, 1);
        if (param.tabName === tab.value) {
          tab.value = "Header";
        }
      }
    }
  });
};

const tabs: Tab[] = [
  { id: "0", label: "Overview" },
  { id: "1", label: "Contents" }
];
const trafficListRef =
  useTemplateRef<InstanceType<typeof TrafficList>>("trafficListRef");
const reqHeaderRef = useTemplateRef<InstanceType<typeof Json>>("reqHeaderRef");
const reqTextRef = useTemplateRef<InstanceType<typeof Text>>("reqTextRef");
const resHeaderRef = useTemplateRef<InstanceType<typeof Json>>("resHeaderRef");
const resTextRef = useTemplateRef<InstanceType<typeof Text>>("resTextRef");

const win = getCurrentWebviewWindow();

// 监听全局搜索的高亮事件
win.listen<ActiveTraffic>("activeTraffic", async (event) => {
  win.setFocus();
  const activeTraffic = event.payload;
  const { keyword, index, position, id, method, sessionId } = activeTraffic;

  if (
    !keyword ||
    !index ||
    !position ||
    !id ||
    !method ||
    !sessionId ||
    !sessionStore.currentSession
  )
    return;

  sessionStore.currentSession = sessionId;

  const currentTrafficId = trafficStore.currentTrafficId.get(
    sessionStore.currentSession
  );
  const trafficId = Number(id);
  // 如果当前高亮的id和当前的id不一致，说明需要更新重新设置更新状态
  if (currentTrafficId !== trafficId) {
    isUpdated.value = false;
  }
  trafficStore.currentTrafficId.set(sessionStore.currentSession, trafficId);

  await waitForCondition(() => isUpdated.value);
  await nextTick();

  activeTab.value = "1";
  // 滚动定位
  trafficListRef.value?.virtualTableRef?.scrollToId(Number(id));

  const word = position.split(" ");
  const pos = word[1];

  switch (pos) {
    case "Header":
      if (method === "request") {
        requestTab.value = "Header";
        await reqHeaderRef.value?.highlight(activeTraffic);
      } else {
        responseTab.value = "Header";
        await resHeaderRef.value?.highlight(activeTraffic);
      }

      break;
    case "Body":
      if (method === "request") {
        requestTab.value = "Text";

        await waitForCondition(() => !!reqTextRef.value);

        reqTextRef.value?.highlight(activeTraffic);
      } else {
        responseTab.value = "Text";

        await waitForCondition(() => !!resTextRef.value);
        resTextRef.value?.highlight(activeTraffic);
      }
      break;
  }
});
</script>

<style scoped>
.split-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.bottom-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  /* overflow-y: hidden; */
}

.panel {
  width: 100%;
  overflow: auto;
}

.panel--top {
  height: 300px;
  background: #f5f5f5;
  flex-shrink: 0;
}

.top-panel-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}

.traffic-list-wrapper {
  flex: 1;
  min-height: 0; /* 重要：防止 flex 子项溢出 */
  overflow-y: auto;
}

.search-container {
  flex-shrink: 0;
  padding: 4px;
  background: #f5f5f5;
  border-top: 1px solid #e8e8e8;
}

.panel--middle {
  flex: 1;
  background: #fff;
  overflow: auto;
  display: flex;
  flex-direction: column;
}

.panel--bottom {
  height: 300px;
  background: #fff;
  flex-shrink: 0;
}

.trafficList {
  text-align: left;
  border: 1px solid #ccc;
  height: 100%;
  width: 100%;
}

/* 滚动条样式 */
.panel::-webkit-scrollbar {
  width: 8px;
}

.panel::-webkit-scrollbar-track {
  background: #f1f3f5;
  border-radius: 10px;
}

.panel::-webkit-scrollbar-thumb {
  background: #adb5bd;
  border-radius: 10px;
}

.panel::-webkit-scrollbar-thumb:hover {
  background: #868e96;
}

.sticky-header {
  position: sticky;
  top: 0;
  background-color: white;
  z-index: 10;
  padding: 2px;
  padding-bottom: 5px;
  border-bottom: 1px solid #f0f0f0;
}

.operate {
  width: 99.7%;
  background-color: #f5f5f5;
  position: absolute;
  bottom: 0px;
  z-index: 10;
  /* overflow: hidden; */
}

.edit-table {
  height: calc(100% - 24px);
}

.traffic-list-wrapper::-webkit-scrollbar {
  width: 8px;
}

.traffic-list-wrapper::-webkit-scrollbar-track {
  background: #f1f3f5;
  border-radius: 10px;
}

.traffic-list-wrapper::-webkit-scrollbar-thumb {
  background: #adb5bd;
  border-radius: 10px;
}

.traffic-list-wrapper::-webkit-scrollbar-thumb:hover {
  background: #868e96;
}
</style>

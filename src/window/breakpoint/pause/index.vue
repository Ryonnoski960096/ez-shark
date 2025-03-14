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

          <Input.Search
            v-if="activeTab === 0"
            v-model:value="searchValue"
            placeholder="Search"
            class="h-32px"
          />
        </div>
        <InfoContent
          v-show="activeTab === 0"
          :orgData="orgData"
          :searchValue="searchValue"
          :activeTab="0"
        />
        <Edit
          :componentMap="componentMap"
          ref="editRef"
          v-model="editTab"
          v-show="activeTab === 1"
          :orgData="orgData"
          :currentKey="currentKey"
          @cancel=""
          @abort=""
          @execute="onExecute"
          :tabs="allTabs"
        />
      </div>
      <div class="operate">
        <div class="w f-l b-#F0F0F0">
          <Segmented
            class="w"
            v-model:value="editTab"
            v-if="activeTab === 1"
            :options="allTabs"
            size="small"
          />
        </div>
        <Space :size="10">
          <Button @click="onCancel" size="small">Cancel</Button>
          <Button @click="onAbort" size="small">Abort</Button>
          <Button @click="notifyExecute" size="small">Execute</Button>
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
import { Button, Input, Segmented, Space } from "ant-design-vue";
import InfoContent from "@/components/layout/main/infoPanel/index.vue";
import Edit, { ExecutePrams } from "./edit/index.vue";

import { ref, onMounted, computed, useTemplateRef, watch } from "vue";
import { Tab } from "@/components/tabs/model";
import { IHeaders, TrafficEditData, useTrafficStore } from "@/stores/traffic";
import { listen } from "@tauri-apps/api/event";
import { allTabs, TabType, trafficModificationAPIParams } from "./edit/model";

import { BreakpointPauseEventName } from "@/enum/event-name";
import { windowInit, windowManager } from "@/stores/WindowManager";
import { onResend, trafficContinue, trafficModification } from "@/api/traffic";
import { isSuccess } from "@/api";
import URL from "@/window/breakpoint/pause/edit/url.vue";
import Hex from "@/window/breakpoint/pause/edit/hex.vue";
import Json from "@/window/breakpoint/pause/edit/json.vue";
import Text from "@/window/breakpoint/pause/edit/text.vue";

// 组件映射
const componentMap: Record<TabType, any> = {
  URL,
  Header: Json,
  Text,
  "JSON Text": Json,
  Hex: Hex
};

windowInit();

const searchValue = ref("");
const trafficStore = useTrafficStore();
const activeTab = ref<number>(0);
const orgData = ref<TrafficEditData<IHeaders> | null>(null);

// 外部控制选中的索引
const currentKey = ref<string | undefined>();

// Panel 和 Splitter 的引用
const leftPanel = ref<InstanceType<typeof Panel> | null>(null);
const rightPanel = ref<InstanceType<typeof Panel> | null>(null);
const splitterRef = ref<InstanceType<typeof Splitter> | null>(null);

const editTab = ref<TabType>("URL");

// 操作
const onCancel = () => {};
const onAbort = () => {};

// 处理选中事件
const handleSelect = ({ key }: { key: string }) => {
  orgData.value = trafficStore.breakpointTrafficMap.get(key) ?? null;
};

// 在挂载后设置元素
onMounted(async () => {
  if (splitterRef.value && leftPanel.value && rightPanel.value) {
    const leftElement = leftPanel.value.$el as HTMLElement;
    const rightElement = rightPanel.value.$el as HTMLElement;
    splitterRef.value.setElements(leftElement, rightElement);
  }

  console.log("onMounted");
  await windowManager.window.emit(BreakpointPauseEventName.OPEN, true);
});

const getTabs = (): Tab[] => {
  const tab = [{ id: 0, label: "Overview" }];
  if (orgData.value?.traffic_type === "request") {
    tab.push({ id: 1, label: "Edit Request" });
  } else if (orgData.value?.traffic_type === "response") {
    tab.push({ id: 1, label: "Edit Response" });
  } else if (orgData.value?.traffic_type === "resend") {
    tab.push({ id: 1, label: "Send Traffic" });
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

  // console.log("新增的键:", addedKeys);
  // console.log("删除的键:", removedKeys);

  // 处理新增的键
  addedKeys.forEach((key) => {
    trafficStore.breakpointTrafficMap.set(key, data[key]);
  });

  // 处理删除的键
  removedKeys.forEach((key) => {
    trafficStore.breakpointTrafficMap.delete(key);
  });

  // console.log("更新断点", e, data, trafficStore.breakpointTrafficMap);
  // for (const key in data) {
  //   trafficStore.breakpointTrafficMap.set(key, data[key]);
  // }
});

const editRef = useTemplateRef("editRef");

// 通知执行
const notifyExecute = () => {
  editRef.value?.onExecute();
};

const getFirstKey = (map: Map<any, any>) => {
  if (map.size === 0) return undefined;
  return map.keys().next().value;
};

// 执行
const onExecute = async ({
  urlParams,
  headerParams,
  textParams
}: ExecutePrams) => {
  if (!currentKey.value) return;

  const params = urlParams.dataSource;

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
    url: `${urlParams.urlEditData.url}${queryString ? "?" + queryString : ""}`,
    method: urlParams.urlEditData.method
  };

  try {
    // const headerParamsObj = ;
    data.modified_headers = JSON.parse(headerParams.value ?? "");
  } catch {}

  const modified_body = textParams.value;

  if (modified_body) {
    data.modified_body = modified_body;
  }
  const trafficType = orgData.value?.traffic_type;

  if (!trafficType) return;
  console.log(data, "data");

  let res;

  switch (trafficType) {
    case "request":
    case "response":
      res = await trafficModification(trafficType, data);
      break;
    case "resend":
      res = await onResend(data);
      console.log(res, "res");
  }
  const trafficStore = useTrafficStore();
  if (isSuccess(res)) {
    console.log(
      res,
      "向父元素发送消息",
      trafficStore.breakpointTrafficMap,
      data.id
    );

    trafficStore.breakpointTrafficMap.delete(data.id);
    console.log("删除成功", trafficStore.breakpointTrafficMap);
    editRef.value?.resetEditParams();
    activeTab.value = 0;
    orgData.value = null;
    currentKey.value = getFirstKey(trafficStore.breakpointTrafficMap);
    if (currentKey.value) handleSelect({ key: currentKey.value });

    if (trafficType === "resend") {
      windowManager.window.emit("sendTraffic", data.id);
    } else {
      windowManager.window.emit("modification", data.id);
      trafficContinue(data.id);
    }
  }
};

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
  /* height: calc(100vh - 32px); */
  /* height: 100%; */
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
  /* height: 32px; */
  background-color: #f5f5f5;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}
</style>

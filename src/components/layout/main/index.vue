<template>
  <div class="split-container">
    <div ref="topPanel" class="panel panel--top">
      <div class="top-panel-container">
        <div class="traffic-list-wrapper">
          <TrafficList class="trafficList" />
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
        <Panel class="w h-100%" v-show="activeTab === 0">
          <div class="sticky-header">
            <Input.Search v-model:value="searchValue" placeholder="Search" />
          </div>
          <InfoContent
            :orgData="orgData"
            :searchValue="searchValue"
            :activeTab="activeTab"
          />
        </Panel>
        <Panel class="pos-relative w h-100% p-2px" v-show="activeTab === 1">
          <!-- <div class="f-col-between-center h-100% w"> -->

          <Edit
            class="edit-table"
            ref="requestEditRef"
            v-model="requestTab"
            HTTPMessages="request"
            :readOnly="true"
            :orgData="orgData"
            :componentMap="componentMap"
            :currentKey="currentKey"
            :tabs="requestTabs"
          />

          <div class="operate">
            <Segmented
              class="w"
              v-model:value="requestTab"
              :options="requestTabs"
              size="small"
            />
          </div>
          <!-- <div class="mt-24px"></div> -->
          <!-- </div> -->
        </Panel>
      </div>
      <Splitter
        v-show="activeTab !== 0"
        ref="splitter2"
        direction="horizontal"
        :min-size="100"
      />
      <div
        v-show="activeTab !== 0"
        ref="bottomPanel"
        style="height: 200px"
        class="panel panel--bottom"
      >
        <Panel class="pos-relative w h-100% p-2px">
          <Edit
            class="edit-table"
            ref="responseEditRef"
            v-model="response"
            HTTPMessages="response"
            :readOnly="true"
            :orgData="orgData"
            :componentMap="componentMap"
            :currentKey="currentKey"
            :tabs="responseTabs"
          />
          <div class="operate">
            <Segmented
              class="w"
              v-model:value="response"
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
import { computed, onMounted, ref, useTemplateRef, watch } from "vue";
import { useEventBus } from "@/hooks";
import {
  IHeaders,
  ITrafficDataDetail,
  useTrafficStore
} from "@/stores/traffic";
import { Tab } from "@/components/tabs/model";
import { Input, InputSearch, Segmented } from "ant-design-vue";
import { TabType } from "@/window/breakpoint/pause/edit/model";
import Edit from "@/window/breakpoint/pause/edit/index.vue";
import Hex from "@/window/breakpoint/pause/edit/hex.vue";
import Json from "@/window/breakpoint/pause/edit/json.vue";
import Text from "@/window/breakpoint/pause/edit/text.vue";
import Panel from "@/components/Panel.vue";
import { useIntervalFn } from "@vueuse/core";
const value = ref("");
const trafficStore = useTrafficStore();

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

// 组件映射
const componentMap: Record<TabType, any> = {
  Text: Text,
  Header: Json,
  "JSON Text": Json,
  Hex: Hex
};
defineOptions({
  name: "Main"
});

const activeTab = ref<number>(0);
const searchValue = ref("");

const currentKey = ref<string | undefined>();
const requestTab = ref<TabType>("Header");
const response = ref<TabType>("Header");

const topPanel = ref<HTMLElement | null>(null);
const middlePanel = ref<HTMLElement | null>(null);
const bottomPanel = ref<HTMLElement | null>(null);
const splitter1 = ref<InstanceType<typeof Splitter> | null>(null);
const splitter2 = ref<InstanceType<typeof Splitter> | null>(null);

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

const { on } = useEventBus();
const orgData = ref<ITrafficDataDetail<IHeaders> | null>(null);
on("change:trafficDetail", (data) => {
  orgData.value = data;
});

const requestEditRef =
  useTemplateRef<InstanceType<typeof Edit>>("requestEditRef");

const responseEditRef =
  useTemplateRef<InstanceType<typeof Edit>>("responseEditRef");

const createTabs = (
  EditRef: ReturnType<typeof useTemplateRef<InstanceType<typeof Edit>>>
) => {
  const tabs: string[] = [];
  if (!EditRef.value) return tabs;
  const { params } = EditRef.value;
  console.log("requestTabs", orgData.value, params);

  if (
    params.headerParams.value.value &&
    params.headerParams.value.value.length > 0
  ) {
    tabs.push("Header");
  }

  if (params.cookieParams.value.value.length > 0) {
    tabs.push("Cookie");
  }

  if (
    params.textParams.value.value &&
    params.textParams.value.value.length > 0
  ) {
    tabs.push("Text");
  }

  if (
    params.JSONParams.value.value &&
    params.JSONParams.value.value.length > 0
  ) {
    try {
      JSON.parse(params.JSONParams.value.value);
      tabs.push("JSON Text");
    } catch {}
  }

  if (params.hexParams.value && params.hexParams.value.length > 0) {
    tabs.push("Hex");
  }

  return tabs;
};

const requestTabs = computed(() => createTabs(requestEditRef));

const responseTabs = computed(() => createTabs(responseEditRef));

const tabs: Tab[] = [
  { id: 0, label: "Overview" },
  { id: 1, label: "Contents" }
];
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

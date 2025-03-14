<template>
  <!-- :style="{ height: `calc(100vh - ${modelValue !== 'URL' ? 105 : 129}px)` }" -->
  <div class="f-col-between-center editor">
    <!-- <div class="w editType"> -->
    <!-- {{ modelValue }} -->
    <KeepAlive>
      <component
        :is="componentMap[modelValue]"
        :key="modelValue"
        :readOnly="readOnly"
        v-if="modelValue === 'URL'"
        v-model:dataSource="urlParams.dataSource"
        v-model:urlEditData="urlParams.urlEditData"
      />
      <!-- :style="{ height: `calc(100% - ${readOnly ? 24 : 60}px)` }" -->
      <!-- style="height: calc(100% - 24px)" -->
      <!-- <EditTable
        :readOnly="readOnly"
        v-else-if="modelValue === 'Header'"
        v-model:dataSource="headerParams.value"
      /> -->
      <component
        :readOnly="readOnly"
        :is="componentMap[modelValue]"
        v-else-if="modelValue === 'Header'"
        v-model:value="headerParams.value"
      />
      <!-- :style="{ height: `calc(100% - ${readOnly ? 24 : 60}px)` }" -->
      <!-- style="height: calc(100% - 24px)" -->
      <EditTable
        :readOnly="readOnly"
        v-else-if="modelValue === 'Cookie'"
        v-model:dataSource="cookieParams.value"
      />
      <component
        :readOnly="readOnly"
        :is="componentMap[modelValue]"
        v-else-if="modelValue === 'Text'"
        v-model:value="textParams.value"
      />
      <component
        :readOnly="readOnly"
        :is="componentMap[modelValue]"
        v-else-if="modelValue === 'JSON Text'"
        v-model:value="JSONParams.value"
      />
      <component
        :readOnly="readOnly"
        :is="componentMap[modelValue]"
        :hexBody="hexParams"
        v-else-if="modelValue === 'Hex'"
      />
    </KeepAlive>
    <!-- </div> -->
  </div>
</template>

<script setup lang="ts">
import { DataItem, allTabs, TabType, urlData } from "./model";

import EditTable from "@/components/EditTable.vue";
import { ref, UnwrapRef, watch } from "vue";
import {
  HexBody,
  IHeaders,
  ITrafficData,
  ITrafficDataDetail,
  TrafficEditData
} from "@/stores/traffic";
// import { deepClone } from "@/utils/tools";
import { useFormat } from "@/components/layout/main/infoPanel/useFormat";
import { HttpRequestHeader } from "ant-design-vue/es/upload/interface";
// 定义 props
const {
  orgData,
  readOnly = false,
  HTTPMessages
} = defineProps<{
  orgData: TrafficEditData<IHeaders> | ITrafficData | null;
  currentKey: string | undefined;
  readOnly?: boolean;
  componentMap: Record<TabType, any>;
  HTTPMessages?: "response" | "request";
}>();

// 定义 model，默认选中第一个 tab
const modelValue = defineModel<TabType>({
  default: allTabs[0]
});

// const orgDataClone = ref<TrafficEditData<IHeaders> | null>(null);
const format = useFormat();

// 初始化所有参数
const urlParams = ref<{
  dataSource: DataItem[];
  urlEditData: urlData;
}>({
  dataSource: [],
  urlEditData: {
    method: undefined,
    url: "",
    params: {},
    httpVersion: undefined
  }
});

const headerParams = ref<{
  value: string | undefined;
}>({
  value: ""
});

const textParams = ref<{
  value: string;
}>({
  value: ""
});

const JSONParams = ref<{
  value: string | undefined;
}>({
  value: ""
});

const cookieParams = ref<{
  value: DataItem[];
}>({
  value: []
});

const hexParams = ref<HexBody[]>([]);

const resetEditParams = () => {
  urlParams.value.dataSource = [];
  urlParams.value.urlEditData = {
    method: undefined,
    url: "",
    params: {},
    httpVersion: undefined
  };
  headerParams.value.value = undefined;
  textParams.value.value = "";
  JSONParams.value.value = undefined;
  cookieParams.value.value = [];
};

// 头部数据处理函数
const headerDataSourceHandel = (header: HttpRequestHeader) => {
  // const dataSource: DataItem[] = [];
  // for (const key in h) {
  //   dataSource.push({
  //     name: key,
  //     value: h[key]
  //   });
  // }
  try {
    return JSON.stringify(header, null, 2);
  } catch (error) {
    console.warn("处理头部数据时出错:", error);

    return "";
  }
};

// 处理 Cookie 的函数
const processCookies = (headerValue: string) => {
  const cookieList: DataItem[] = [];
  if (!headerValue) return cookieList;

  const cookies = headerValue.split(";").filter(Boolean);
  cookies.forEach((cookie) => {
    const [name, value] = cookie
      .trim()
      .split("=")
      .map((item) => item.trim());
    if (name) {
      cookieList.push({ name, value: value || "" });
    }
  });

  return cookieList;
};

// 监听原始数据变化
watch(
  () => orgData,
  (newVal) => {
    if (!newVal) {
      // 重置所有参数为初始状态
      urlParams.value.dataSource = [];
      urlParams.value.urlEditData = {
        method: undefined,
        url: "",
        params: {},
        httpVersion: undefined
      };
      headerParams.value.value = undefined;
      textParams.value.value = "";
      JSONParams.value.value = undefined;
      cookieParams.value.value = [];
      hexParams.value = [];
      return;
    }

    // orgDataClone.value = deepClone(newVal);

    // 处理 URL 参数
    if (newVal.traffic && newVal.traffic.uri) {
      try {
        // 解析 URL
        const url = new URL(newVal.traffic.uri);

        const hasExplicitPort = /:\d+/.test(newVal.traffic.uri);

        urlParams.value.urlEditData = {
          method: newVal.traffic.method,
          url: `${url.protocol}//${url.hostname}${
            hasExplicitPort
              ? ":" + (url.port || (url.protocol === "http:" ? "80" : "443"))
              : ""
          }${url.pathname}`,
          params: Object.fromEntries(url.searchParams),
          httpVersion: newVal.traffic.http_version
        };

        // 构建 dataSource
        urlParams.value.dataSource = Object.entries(
          urlParams.value.urlEditData.params
        ).map(([name, value]) => ({
          name,
          value: value as string
        }));
      } catch (error) {
        console.error("处理 URL 时出错:", error);
        urlParams.value.dataSource = [];
        urlParams.value.urlEditData = {
          method: undefined,
          url: "",
          params: {},
          httpVersion: undefined
        };
      }
    } else {
      // 如果没有 URL 信息，重置为空
      urlParams.value.dataSource = [];
      urlParams.value.urlEditData = {
        method: undefined,
        url: "",
        params: {},
        httpVersion: undefined
      };
    }

    // 处理 header
    if (newVal.traffic && newVal.traffic.req_headers) {
      try {
        const h = format.formatHeaders(newVal.traffic.req_headers);

        headerParams.value.value = headerDataSourceHandel(h);

        // 处理 Cookie
        const cookieHeader = h.cookie;
        if (cookieHeader) {
          cookieParams.value.value = processCookies(cookieHeader);
        } else {
          // 如果没有 Cookie，重置为空
          cookieParams.value.value = [];
        }
      } catch (error) {
        console.error("处理头部时出错:", error);
        headerParams.value.value = undefined;
        cookieParams.value.value = [];
      }
    } else {
      // 如果没有 headers 信息，重置为空
      headerParams.value.value = undefined;
      cookieParams.value.value = [];
    }

    if (!HTTPMessages) {
      // 处理请求体
      if (
        (newVal as TrafficEditData<IHeaders>).body &&
        (newVal as TrafficEditData<IHeaders>).body?.value
      ) {
        try {
          const bodyValue = (newVal as TrafficEditData<IHeaders>).body?.value;
          if (!bodyValue) return;
          // 处理文本
          textParams.value.value =
            typeof bodyValue === "string"
              ? bodyValue
              : JSON.stringify(bodyValue, null, 2);
          // console.log(
          //   "textParams.value.value , 更新更新更新更新",
          //   textParams.value.value
          // );
          // 处理 JSON
          JSONParams.value.value = bodyValue;
        } catch (error) {
          console.error("处理请求体时出错:", error);
          textParams.value.value = "";
          JSONParams.value.value = undefined;
        }
      } else {
        textParams.value.value = "";
        JSONParams.value.value = undefined;
        // console.log(
        //   "textParams.value.value , 更新更新更新更新",
        //   textParams.value.value
        // );
      }
    } else if (HTTPMessages) {
      const httpMsg =
        HTTPMessages === "request"
          ? "req_body"
          : HTTPMessages === "response"
            ? "res_body"
            : undefined;
      if (!httpMsg) {
        textParams.value.value = "";
        // console.log(
        //   "textParams.value.value , 更新更新更新更新",
        //   textParams.value.value
        // );
        JSONParams.value.value = undefined;
        return;
      }

      if ((newVal as ITrafficDataDetail<IHeaders>)[httpMsg]) {
        try {
          const bodyValue = (newVal as ITrafficDataDetail<IHeaders>)[httpMsg]
            ?.value;
          if (!bodyValue) return;
          // 处理文本
          textParams.value.value =
            typeof bodyValue === "string"
              ? bodyValue
              : JSON.stringify(bodyValue, null, 2);

          // 处理 JSON
          JSONParams.value.value = bodyValue;
          // console.log(
          //   "textParams.value.value , 更新更新更新更新",
          //   textParams.value.value
          // );
        } catch (error) {
          console.error("处理请求体时出错:", error);
          textParams.value.value = "";
          JSONParams.value.value = undefined;
          // console.log(
          //   "textParams.value.value , 更新更新更新更新",
          //   textParams.value.value
          // );
        }
      }
    } else {
      // 如果没有请求体信息，重置为空
      textParams.value.value = "";
      console.log(
        "textParams.value.value , 更新更新更新更新",
        textParams.value.value
      );
      JSONParams.value.value = undefined;
    }
    // 处理 Hex
    if (newVal.traffic) {
      if (!newVal.traffic.req_body_hex && !newVal.traffic.res_body_hex) return;

      let httpMsg =
        HTTPMessages === "request"
          ? "req_body_hex"
          : HTTPMessages === "response"
            ? "res_body_hex"
            : undefined;

      // console.log("1", httpMsg);
      if (!httpMsg) {
        httpMsg =
          (newVal as TrafficEditData).traffic_type === "request"
            ? "req_body_hex"
            : (newVal as TrafficEditData).traffic_type === "response"
              ? "res_body_hex"
              : undefined;
        if (!httpMsg) {
          hexParams.value = [];
          return;
        }
      }
      const hexBody = (newVal.traffic as Record<string, any>)[httpMsg];
      // console.log(hexBody);
      if (hexBody) {
        hexParams.value = hexBody;
      }
    }
  },
  { immediate: true }
);

// 监听 Cookie 变化，同步到 Header
// 添加一个标记来控制是否需要触发更新
let isUpdatingFromCookie = false;
let isUpdatingFromHeader = false;

// 监听 cookie 变化并更新到 header
watch(
  () => cookieParams.value.value,
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
      console.log(cookieStr);

      // 更新 Header 中的 Cookie
      const headerObj = JSON.parse(headerParams.value.value ?? "");
      headerObj.cookie = cookieStr.length !== 0 ? cookieStr : undefined;
      headerParams.value.value = JSON.stringify(headerObj, null, 2);
    } catch {}
    isUpdatingFromCookie = false;
  },
  { deep: true }
);

// 监听 header 变化并更新到 cookie
watch(
  () => headerParams.value.value,
  (newValue) => {
    // 如果是从 cookie 更新触发的，则不需要再更新回 cookie
    if (isUpdatingFromCookie) {
      return;
    }

    try {
      isUpdatingFromHeader = true;
      const headerCookie = JSON.parse(newValue ?? "").cookie;
      if (headerCookie) {
        // 清空现有的 cookie
        cookieParams.value.value = [];

        const cookieArr = headerCookie.split(";");
        for (const cookie of cookieArr) {
          const [name, value] = cookie.split("=");
          cookieParams.value.value.push({
            name: name.trim(),
            value: value.trim()
          });
        }
      }
      isUpdatingFromHeader = false;
    } catch {
      isUpdatingFromHeader = false;
    }
  },
  { deep: true }
);

watch(
  () => [textParams.value.value, JSONParams.value.value],
  ([newTextValue, newJsonValue], [oldTextValue]) => {
    // 安全的 JSON 解析函数
    const safeJsonParse = (value: string) => {
      try {
        return JSON.parse(value);
      } catch {
        return value;
      }
    };

    // 安全的 JSON 字符串化函数
    const safeJsonStringify = (value: any): string => {
      try {
        return typeof value === "string"
          ? value
          : JSON.stringify(value, null, 2);
      } catch {
        return String(value);
      }
    };

    // 处理空值
    if (!newTextValue) {
      newTextValue = "";
    }

    // 尝试解析 text 为 JSON
    const parsedText = safeJsonStringify(safeJsonParse(newTextValue));
    if (parsedText !== oldTextValue) {
      JSONParams.value.value = parsedText;
    }

    // 格式化 JSON 为字符串
    const formattedText = safeJsonStringify(newJsonValue);

    if (formattedText !== oldTextValue) {
      textParams.value.value = formattedText;
    }
  },
  { deep: true }
);
export interface ExecutePrams {
  urlParams: UnwrapRef<typeof urlParams>;
  headerParams: UnwrapRef<typeof headerParams>;
  textParams: UnwrapRef<typeof textParams>;
  jsonParams: UnwrapRef<typeof JSONParams>;
  cookieParams: UnwrapRef<typeof cookieParams>;
}

const emit = defineEmits<{
  (event: "cancel", ...args: any[]): void;
  (event: "abort", ...args: any[]): void;
  (event: "execute", params: ExecutePrams): void;
}>();

const onExecute = async () => {
  emit("execute", {
    urlParams: urlParams.value,
    headerParams: headerParams.value,
    textParams: textParams.value,
    jsonParams: JSONParams.value,
    cookieParams: cookieParams.value
  });
};

defineExpose({
  resetEditParams,
  onExecute,
  params: {
    urlParams,
    headerParams,
    textParams,
    JSONParams,
    cookieParams,
    hexParams
  }
});
</script>

<style scoped>
.editor {
  width: 100%;
  height: 100%;
  overflow-x: hidden;
}

.editType {
  border: 1px ridge #89858585;
  height: calc(100% - 36px);
}
</style>

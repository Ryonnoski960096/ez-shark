<template>
  <div class="info-content">
    <Tree
      :expanded-keys="expandedKeys"
      :selected-keys="[active]"
      :auto-expand-parent="autoExpandParent"
      :tree-data="treeData"
      @expand="onExpand"
      @select="onSelect"
    >
      <template #title="{ value, key, children }">
        <!-- @dblclick.stop="toggleEdit(key)" -->
        <div
          class="tree-node-content"
          @contextmenu.prevent="oncontextmenu($event, key, value)"
          :ref="
            (el) => {
              if (el) itemRefs[key] = el as HTMLElement;
            }
          "
        >
          <span class="field">
            <template
              v-if="
                searchValue &&
                getFieldName(key)
                  .toLowerCase()
                  .includes(searchValue.toLowerCase())
              "
            >
              <component :is="getHighlightedText(key)" />
            </template>
            <template v-else>{{ getFieldName(key) }}</template>
            :
          </span>
          <span v-if="isEdit(key, value, children)" class="input-wrapper">
            <Textarea
              :ref="(el) => textareaRefs.set(key, el)"
              v-model:value="editValue"
              @input="input(key)"
              size="small"
              :auto-size="{ minRows: 1 }"
              @keydown.enter="
                (e) => {
                  if (!e.shiftKey) {
                    e.preventDefault();
                    editorKey = '';
                  }
                }
              "
            />
          </span>
          <span v-else class="value">
            <component
              v-if="
                searchValue &&
                formatValue(value)
                  .toLowerCase()
                  .includes(searchValue.toLowerCase())
              "
              :is="highlightText(formatValue(value), searchValue)"
            />
            <template v-else>
              {{ truncateText(formatValue(value)) }}
            </template>
          </span>
        </div>
      </template>
    </Tree>
    <DebugToolbar id="debugToolbar" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, h } from "vue";
import type { Key } from "ant-design-vue/es/_util/type";
import { Tree, Textarea } from "ant-design-vue";
import ContextMenu from "@imengyu/vue3-context-menu";

import DebugToolbar from "@/components/debugToolbar/index.vue";
import {
  TrafficEditData,
  useTrafficStore,
  type IHeaders,
  type ITrafficDataDetail
} from "@/stores/traffic";
import { HTTPMeta, Method, Request, Response } from "./model";
import { useFormat } from "./useFormat";
import { HttpRequestHeader } from "ant-design-vue/es/upload/interface";
import { copyContent } from "@/utils/tools";

// 定义树节点接口
interface TreeNode {
  title: any;
  key: Key;
  value?: any;
  children?: TreeNode[];
}

const { activeTab, searchValue, orgData } = defineProps<{
  activeTab: number;
  searchValue: string;
  orgData: ITrafficDataDetail<IHeaders> | TrafficEditData<IHeaders> | null;
}>();

const editValue = ref("");
const active = ref<Key>("");
const editorKey = ref("");
const itemRefs: Record<string, HTMLElement> = {};
const request = ref<HTTPMeta>();
const expandedKeys = ref<Key[]>([]);
const autoExpandParent = ref(true);
const textareaRefs = ref(new Map());

const convertToTreeData = (obj: HTTPMeta, parentKey = ""): TreeNode[] => {
  if (!obj || typeof obj !== "object") return [];

  // 处理对象
  return Object.entries(obj)
    .filter(([_, value]) => {
      // 过滤掉 null、undefined、空字符串、空数组、空对象
      if (value === null || value === undefined) return false;

      if (typeof value === "string" && value.trim() === "") return false;

      if (Array.isArray(value) && value.length === 0) return false;

      if (typeof value === "object" && value !== null) {
        return Object.keys(value).length > 0;
      }

      return true;
    })
    .map(([key, value]) => {
      const currentKey = parentKey ? `${parentKey}.${key}` : key;

      // 如果值是对象（不包括数组）
      if (
        typeof value === "object" &&
        !Array.isArray(value) &&
        value !== null
      ) {
        const children = convertToTreeData(value, currentKey);

        // 只有当有子节点时才返回带子节点的对象
        return {
          title: null,
          key: currentKey,
          value: value,
          ...(children.length > 0 ? { children } : {})
        };
      }

      // 基本数据类型
      return {
        title: null,
        key: currentKey,
        value: value
      };
    });
};

const formatValue = (value: any): string => {
  if (value === null) return "null";
  if (value === undefined) return "undefined";
  if (typeof value === "string") return `${value}`;
  if (typeof value === "number" || typeof value === "boolean")
    return String(value);
  if (Array.isArray(value)) return `Array(${value.length})`;
  if (typeof value === "object") {
    const str = JSON.stringify(value).replace(/"([^"]+)":/g, "$1:");
    return str.length > 60 ? str.slice(0, 30) + " ... " + str.slice(-30) : str;
  }
  return String(value);
};

const treeData = computed(() => {
  if (request.value) {
    return convertToTreeData(request.value);
  } else {
    return [];
  }
});

const trafficStore = useTrafficStore();

const input = (key: string) => {
  if (key === editorKey.value) {
    const keys = editorKey.value.split(".");

    const setNestedValue = (obj: any, keys: string[], value: any) => {
      return keys.reduce((acc, key, index) => {
        if (index === keys.length - 1) {
          acc[key] = value;
        } else {
          acc[key] = acc[key] || {};
        }
        return acc[key];
      }, obj);
    };

    setNestedValue(request.value, keys, editValue.value);

    try {
      trafficStore.trafficEditStatusMap.set(waitProc.value?.traffic.gid!, true);
    } catch (e) {
      console.error("error", waitProc.value);
    }
  }
};

const isEdit = (key: string, value: string, children?: any[]) => {
  const isEditStatus =
    editorKey.value === key && (children?.length === 0 || !children);
  if (isEditStatus) {
    editValue.value = value;
    try {
      textareaRefs.value.get(key).focus();
    } finally {
      return true;
    }
  }
  return false;
};

const highlightText = (text: string, searchValue: string) => {
  if (!searchValue) return text;

  const lowerText = text.toLowerCase();
  const lowerSearchValue = searchValue.toLowerCase();
  const startIndex = lowerText.indexOf(lowerSearchValue);

  if (startIndex === -1) return text;

  return h("span", [
    h("span", text.substring(0, startIndex)),
    h(
      "span",
      { class: "highlight" },
      text.substring(startIndex, startIndex + searchValue.length)
    ),
    h("span", text.substring(startIndex + searchValue.length))
  ]);
};

const getHighlightedText = (key: string | number) => {
  const fieldName = getFieldName(String(key));
  const formattedValue = formatValue(getValueByPath(String(key)));

  // 优先检查字段名
  const lowerCaseFieldName = fieldName.toLowerCase();
  const lowerCaseSearchValue = searchValue.toLowerCase();

  if (lowerCaseFieldName.includes(lowerCaseSearchValue)) {
    return highlightText(fieldName, searchValue);
  }

  // 如果字段名不匹配，检查值
  const lowerCaseFormattedValue = formattedValue.toLowerCase();
  if (lowerCaseFormattedValue.includes(lowerCaseSearchValue)) {
    return highlightText(formattedValue, searchValue);
  }

  return fieldName;
};
// 获取父级key
const getParentKey = (key: Key, tree: TreeNode[]): Key | undefined => {
  let parentKey;
  if (!tree) return undefined;

  for (let i = 0; i < tree.length; i++) {
    const node = tree[i];
    if (node.children) {
      if (node.children.some((item) => item.key === key)) {
        parentKey = node.key;
      } else if (getParentKey(key, node.children)) {
        parentKey = getParentKey(key, node.children);
      }
    }
  }
  return parentKey;
};

const getFieldName = (key: string) => {
  const parts = key.split(".");
  return parts[parts.length - 1].replace(/\[\d+\]$/, "");
};

const getValueByPath = (path: string) => {
  return path
    .split(/[.\[\]]/)
    .filter(Boolean)
    .reduce((obj: any, key) => obj?.[key], request.value);
};

const onSelect = (selectedKeys: Key[]) => {
  if (selectedKeys.length > 0) {
    active.value = selectedKeys[0];
  }
};

const onExpand = (keys: Key[]) => {
  expandedKeys.value = keys;
  autoExpandParent.value = false;
};

function oncontextmenu(e: MouseEvent, key: string, value: any) {
  e.preventDefault();
  active.value = key;

  ContextMenu.showContextMenu({
    x: e.x,
    y: e.y,
    items: [
      {
        label: "复制",
        onClick: () => {
          console.log("复制", key, value);
          if (typeof value === "string") copyContent(value);
          else {
            copyContent(JSON.stringify(value));
          }
        }
      }
    ]
  });
}

// 生成扁平化的数据列表用于搜索
const generateList = (data: TreeNode[]) => {
  const dataList: TreeNode[] = [];
  const traverse = (nodes: TreeNode[]) => {
    nodes?.forEach((node) => {
      dataList.push(node);
      if (node.children) {
        traverse(node.children);
      }
    });
  };
  traverse(data);
  return dataList;
};

// 监听搜索值变化
watch(
  () => searchValue,
  (value) => {
    if (!value) {
      expandedKeys.value = [];
      return;
    }

    const dataList = generateList(treeData.value);
    const lowerCaseValue = value.toLowerCase(); // 转换搜索值为小写

    const expanded = dataList
      .map((item) => {
        const fieldName = getFieldName(item.key.toString());
        const formattedValue = formatValue(item.value);

        // 同时搜索键和值
        const matchesField =
          fieldName && fieldName.toLowerCase().includes(lowerCaseValue);
        const matchesValue =
          formattedValue &&
          formattedValue.toLowerCase().includes(lowerCaseValue);

        if (matchesField || matchesValue) {
          return getParentKey(item.key, treeData.value);
        }
        return null;
      })
      .filter((item, i, self): item is Key => {
        return item !== null && item !== undefined && self.indexOf(item) === i;
      });

    expandedKeys.value = expanded;
    autoExpandParent.value = true;
  }
);

const { formatHeaders, truncateText } = useFormat();

// 格式化数据
const formatData = (
  data: ITrafficDataDetail<HttpRequestHeader> | undefined
) => {
  if (!data) return (request.value = undefined);
  console.log(data);
  const parseAndFormat = (value: any) => {
    try {
      return JSON.parse(value);
    } catch {
      return value || null;
    }
  };

  const req: Request = {
    RequestHeaders: data.traffic.req_headers,
    RequestBody: parseAndFormat(data.req_body?.value)
  };

  const res: Response = {
    ResponseHeaders: data.traffic.res_headers,
    ResponseBody: parseAndFormat(data.res_body?.value)
  };

  const body =
    parseAndFormat((data as unknown as TrafficEditData).body?.value) ??
    undefined;

  switch (activeTab) {
    case 0: {
      request.value = {
        Protocol: data.traffic.http_version,
        Method: data.traffic.method as Method,
        Url: data.traffic.uri,
        Status: "暂无",
        ResponseCode: data.traffic.status,
        Request: req,
        Response: res,
        Body: body
      };
      console.log("request.value", request.value);
      break;
    }
    case 1: {
      request.value = req;
      break;
    }
    case 2: {
      request.value = res;
      break;
    }
  }
};

const waitProc = ref<
  | ITrafficDataDetail<HttpRequestHeader | IHeaders>
  | TrafficEditData<HttpRequestHeader | IHeaders>
>();

// 展开根节点
const expandRoot = () => {
  // 默认展开第一级节点
  expandedKeys.value = Object.keys(request.value ?? {}) as Key[];
};

const reLoad = () => {
  console.log("reLoad", waitProc.value);

  formatData(waitProc.value as ITrafficDataDetail<HttpRequestHeader>);
  expandRoot();
};

defineExpose({
  reLoad
});

watch(
  () => orgData,
  (newData) => {
    if (!newData) {
      waitProc.value = undefined;
    } else {
      waitProc.value = {
        ...newData,
        traffic: {
          ...newData.traffic,
          req_headers: formatHeaders(newData.traffic.req_headers),
          res_headers: formatHeaders(newData.traffic.res_headers)
        }
      };
    }
    reLoad();
  }
);

watch(
  () => activeTab,
  () => {
    reLoad();
  }
);
</script>

<style scoped>
.info-content {
  width: 100%;
  /* height: calc(100vh - 140px); */
  background-color: #fff;
  padding: 0 16px 16px 16px;
  overflow-y: auto;
}

.tree-node-content {
  display: flex;
  align-items: center;
  gap: 4px;
  font-family: Menlo, Monaco, Consolas, "Courier New", monospace;
  font-size: 15px;
  padding: 0px 4px;
  border-radius: 4px;
  width: 100%;
}

:deep(.ant-tree-node-selected) {
  background: transparent !important;
}

:deep(.ant-tree-node-selected) .tree-node-content {
  background-color: #428bca;
  color: #fff;
}

.field {
  font-size: 14px;
  font-weight: 600;
  flex-shrink: 0;
}

.highlight {
  color: #f50;
}

:deep(.ant-tree-node-selected) .highlight {
  color: #fff;
  background: #f50;
}

.value {
  flex-grow: 1;
  white-space: pre-wrap; /* 允许换行，保留空格和换行符 */
  word-break: break-all; /* 允许在任意字符间换行 */
  min-width: 0; /* 防止flex子项溢出 */
}
.input-wrapper {
  flex-grow: 1;
  display: flex;
}

:deep(.ant-input) {
  font-family: inherit;
  font-size: inherit;
}

:deep(#debugToolbar) {
  position: fixed;
  bottom: 30px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 1000;
  background-color: white;
  padding: 5px 10px;
  box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.1);
  border-radius: 5px;
}

:deep(.ant-tree-node-content-wrapper) {
  display: flex;
  flex: 1;
}

:deep(.ant-tree-title) {
  flex: 1;
  padding: 0;
  width: 100%;
}

:deep(.ant-tree-treenode.ant-tree-treenode-switcher-close),
:deep(.ant-tree-treenode.ant-tree-treenode-switcher-open) {
  width: 100%;
}

/* 减小展开图标的大小和间距 */
:deep(.ant-tree-switcher) {
  width: 14px !important;
  height: 24px !important;
  line-height: 24px !important;
}

/* 图标与文本的对齐 */
:deep(.ant-tree-switcher-icon) {
  vertical-align: middle;
}
</style>

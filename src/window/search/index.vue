<template>
  <Page>
    <ElForm
      size="small"
      label-width="auto"
      @submit.prevent="onSearch"
      :model="findData"
    >
      <ElFormItem label="Text to find">
        <ElInput v-model="findData.text" placeholder="Please input keyword" />
      </ElFormItem>
      <ElForm size="small" inline :model="findData.position">
        <ElFormItem label="Search position">
          <ElCheckbox v-model="findData.position.request_url"
            >Request URL</ElCheckbox
          >
          <ElCheckbox v-model="findData.position.request_header"
            >Request Header</ElCheckbox
          >
          <ElCheckbox v-model="findData.position.request_body"
            >Request Body</ElCheckbox
          >
          <ElCheckbox v-model="findData.position.response_header"
            >Response Header</ElCheckbox
          >
          <ElCheckbox v-model="findData.position.response_body"
            >Response Body</ElCheckbox
          >
        </ElFormItem>
      </ElForm>
      <ElButton native-type="submit" type="success">Find</ElButton>
    </ElForm>

    <div class="table-wrapper mt-8px">
      <ElTable
        ref="tableRef"
        :data="currentTableData"
        style="width: 100%"
        height="450"
        :header-cell-style="{ background: '#f5f7fa80' }"
        @row-click="handleRowClick"
        :row-style="getRowStyle"
      >
        <ElTableColumn
          prop="url"
          label="URL"
          min-width="300"
          :show-overflow-tooltip="{
            showAfter: 500
          }"
        >
          <template #default="{ row }">
            <div
              class="url-cell"
              :title="row.url"
              :class="{ 'url-active': expandedRowsMap[row.id] }"
              @click.right="rightClickRow($event, row)"
            >
              {{ row.url }}
            </div>
          </template>
        </ElTableColumn>
        <ElTableColumn type="expand" width="1">
          <template #default="{ row }">
            <div class="expanded-content-wrapper">
              <div class="expanded-content">
                <div v-for="item in row.children" :key="item.id">
                  <ExpandRow :rowData="item" />
                </div>
              </div>
            </div>
          </template>
        </ElTableColumn>
      </ElTable>

      <div class="pagination-container">
        <ElPagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :total="totalItems"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </div>
  </Page>
</template>

<script setup lang="tsx">
import { ref, computed, reactive } from "vue";
import Page from "@/components/Page.vue";
import {
  ElButton,
  ElCheckbox,
  ElForm,
  ElFormItem,
  ElInput,
  ElTable,
  ElTableColumn,
  ElPagination
} from "element-plus";
import type { SearchQuery } from "@/api/search/model";
import { search } from "@/api/search";
import { windowInit } from "@/stores/WindowManager";
import { generateMarkStyle } from "@/utils/format";
import { type ActiveTraffic } from "@/utils/eventBus";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import ContextMenu from "@imengyu/vue3-context-menu";
import { copyContent } from "@/utils/tools";
import { useSettingStore } from "@/stores/settings";
import type { JSX } from "vue/jsx-runtime";

// 窗口初始化
windowInit();

interface TableDataChildren {
  id: string;
  position: string;
  content: JSX.Element | string;
  keyword_byte_index: number;
}

interface TableData {
  id: string;
  url: string;
  children: TableDataChildren[];
}

const tableRef = ref<InstanceType<typeof ElTable> | null>(null);
const tableData = ref<TableData[]>([]);
const currentPage = ref(1);
const pageSize = ref(10);
const keyword = ref("");
const expandedRowsMap = reactive<Record<string, boolean>>({});

// 计算总条目数
const totalItems = computed(() => tableData.value.length);

// 计算当前页显示的数据
const currentTableData = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  const end = start + pageSize.value;
  return tableData.value.slice(start, end);
});

const findData = ref<SearchQuery>({
  text: "",
  position: {
    request_url: true,
    request_header: true,
    request_body: true,
    response_header: true,
    response_body: true
  }
});

const currentSession = ref<string | undefined>("");

const onSearch = async () => {
  const settingStore = useSettingStore();
  const store = await settingStore.store;
  currentSession.value = await store.get<string>("currentSession");
  const res = await search(findData.value, currentSession.value ?? "");

  keyword.value = res.text;

  const highlightFirstOccurrence = (
    text: string,
    keyword: string,
    index: number
  ) => {
    if (!keyword) return text;
    const keywordLengthIndex = index + keyword.length;
    return (
      <>
        {text.slice(index - 10, index)}
        <span class="ml-3px mr-3px" style={generateMarkStyle({})}>
          {text.slice(index, keywordLengthIndex)}
        </span>
        {text.slice(keywordLengthIndex, keywordLengthIndex + 10)}
      </>
    );
  };

  // 基于 keyword_byte_index 数组生成 children
  tableData.value = res.search_data.map((item) => ({
    id: item.id,
    url: item.url,
    children: item.search_item.flatMap((searchItem, searchItemIndex) => {
      const searchItemContent = searchItem.content;

      // 基于 keyword_byte_index 数组的长度创建子项
      return searchItem.keyword_byte_index.map((byteIndex, indexPos) => {
        const content = highlightFirstOccurrence(
          searchItemContent,
          keyword.value,
          byteIndex
        );
        return {
          id: `${item.id}-${searchItem.position}-${byteIndex}-${searchItemIndex}-${indexPos}`,
          position: searchItem.position,
          content,
          keyword_byte_index: byteIndex
        };
      });
    })
  }));

  currentPage.value = 1; // 搜索后重置到第一页

  // 重置展开状态
  Object.keys(expandedRowsMap).forEach((key) => {
    expandedRowsMap[key] = false;
  });
};

// 处理行点击事件，切换行的展开状态
const handleRowClick = (row: TableData) => {
  if (tableRef.value) {
    const isCurrentlyExpanded = !!expandedRowsMap[row.id];
    expandedRowsMap[row.id] = !isCurrentlyExpanded;
    tableRef.value.toggleRowExpansion(row);
  }
};

// 根据行展开状态返回行样式
const getRowStyle = ({ row }: { row: TableData }) => {
  if (expandedRowsMap[row.id]) {
    return {
      backgroundColor: "#f0f9ff",
      borderLeft: "3px solid #409eff"
    };
  }
  return {};
};

// 分页处理函数
const handleSizeChange = (val: number) => {
  pageSize.value = val;
  currentPage.value = 1;
};

const handleCurrentChange = (val: number) => {
  currentPage.value = val;
};

// ExpandRow 组件
const ExpandRow = ({ rowData }: { rowData: TableDataChildren }) => {
  const content = rowData.content;
  const position = rowData.position;

  return (
    <div
      onMousedown={(e) => {
        if (e.detail > 1) {
          e.preventDefault();
          onDblclick({
            id: rowData.id,
            keyword: keyword.value,
            position: rowData.position,
            index: rowData.keyword_byte_index,
            sessionId: currentSession.value ?? ""
          });
        }
      }}
      class="item-row cp"
    >
      <span class="mr-10px">{position}:</span>
      <span>{content}</span>
    </div>
  );
};

const win = getCurrentWebviewWindow();

const onDblclick = (data: ActiveTraffic) => {
  data.id = data.id.split("-")[0];
  const payload: ActiveTraffic = {
    ...data,
    method: data.position.includes("Request") ? "request" : "response"
  };
  win.emitTo("main", "activeTraffic", payload);
};

const rightClickRow = (e: MouseEvent, row: TableData) => {
  e.preventDefault();
  ContextMenu.showContextMenu({
    x: e.clientX,
    y: e.clientY,
    items: [
      {
        label: "Copy Url",
        onClick: () => {
          copyContent(row.url);
        }
      },
      {
        label: "Go To",
        onClick: () => {
          win.emitTo("main", "activeTraffic", {
            method: "request",
            keyword: keyword.value,
            index: -1,
            position: "URL",
            id: row.id,
            sessionId: currentSession.value ?? ""
          });
        }
      }
    ]
  });
};
</script>

<style scoped>
.table-wrapper {
  height: calc(100vh - 180px);
  display: flex;
  flex-direction: column;
}

.el-table {
  flex: 1;
}

/* URL单元格样式 */
.url-cell {
  cursor: pointer;
  color: #606266;
  width: 100%;
  height: 100%;
  padding: 5px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.url-cell:hover {
  color: #409eff;
}

/* 激活状态的URL */
.url-active {
  color: #409eff !important;
  font-weight: 500;
}

/* 展开内容的包装容器 */
.expanded-content-wrapper {
  width: 100%;
  padding: 0;
  margin: 0;
  box-sizing: border-box;
}

/* 展开的内容 */
.expanded-content {
  max-height: 300px;
  overflow-y: auto;
  width: calc(100% - 20px);
  margin-right: 20px; /* 给表格滚动条留出空间 */
  padding: 0;
}

.item-row {
  padding: 5px 0 5px 36px;
  border-top: 1px solid #e4e7ed;
}

.pagination-container {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}

/* 修改表格展开行样式 */
:deep(.el-table__expanded-cell) {
  padding: 10px 20px !important;
}

/* 隐藏展开图标列 */
:deep(.el-table__expand-column .cell) {
  display: none;
}

/* 确保表格和展开内容的滚动条不重叠 */
:deep(.el-table__body-wrapper) {
  overflow-y: auto !important;
  overflow-x: auto !important;
}

/* 防止表格单元格内容换行 */
:deep(.el-table .cell) {
  white-space: nowrap;
}

/* 美化滚动条样式 */
.expanded-content::-webkit-scrollbar {
  width: 7px;
  height: 7px;
}

.expanded-content::-webkit-scrollbar-thumb {
  background-color: #c0c4cc;
  border-radius: 3px;
}

.expanded-content::-webkit-scrollbar-track {
  background-color: #f5f7fa;
}

:deep(.el-table__body-wrapper::-webkit-scrollbar-thumb) {
  background-color: #c0c4cc;
  border-radius: 4px;
}

:deep(.el-table__body-wrapper::-webkit-scrollbar-track) {
  background-color: #f5f7fa;
}

/* 设置表格行高 */
:deep(.el-table__row) {
  height: 28px;
}
:deep(.el-table__cell) {
  padding: 0 !important;
  height: 28px;
  line-height: 28px;
}

/* 防止高亮行被 hover 效果覆盖 */
:deep(.el-table__body tr:hover > td) {
  background-color: inherit;
}

/* 展开行项目的样式 */
.item-row {
  padding: 1px 0 1px 36px;
  border-top: 1px solid #e4e7ed;
  transition: all 0.3s ease;
  position: relative;
}

/* 悬停效果 */
.item-row:hover {
  background-color: #ecf5ff;
  padding-left: 40px;
}

/* 悬停时添加左侧指示条 */
.item-row:hover::before {
  content: "";
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  width: 3px;
  background-color: #409eff;
}

/* 鼠标悬停时改变文本颜色 */
.item-row:hover span {
  color: #409eff;
}
</style>

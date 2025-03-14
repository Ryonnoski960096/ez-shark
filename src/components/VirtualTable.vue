<template>
  <div ref="containerRef" class="virtual-table-container">
    <!-- 表头 -->
    <div class="table-header">
      <div
        v-for="(column, index) in localColumns"
        :key="column.key"
        class="header-cell"
        :class="{
          'flex-grow': index === localColumns.length - 1,
          sortable: true,
          sorted: sortState.key === column.key
        }"
        :style="column.isFlex ? {} : { width: `${column.width}px` }"
        @click="handleSort(column)"
      >
        <div class="header-content">
          <b>{{ column.title }}</b>
          <span class="sort-icon" v-if="sortState.key === column.key">
            {{ sortState.direction === "asc" ? "↑" : "↓" }}
          </span>
        </div>
        <div
          v-if="index < localColumns.length - 1"
          class="column-resizer"
          @mousedown.stop="startResize($event, index)"
        ></div>
      </div>
    </div>

    <!-- 滚动容器 -->
    <div ref="scrollContainerRef" class="scroll-container" @scroll="onScroll">
      <!-- 撑开高度的容器 -->
      <div class="scroll-height" :style="{ height: `${totalHeight}px` }">
        <!-- 可视区域 -->
        <div
          class="virtual-list cp"
          :style="{
            transform: `translateY(${startOffset}px)`
          }"
        >
          <div
            v-for="item in visibleData"
            :key="item.id"
            class="virtual-row"
            :style="{ height: `${rowHeight}px` }"
            :class="{ 'row-active': active === item.id }"
            @click.stop="onCellClick(item)"
            @click.right="onContextMenu($event, item)"
          >
            <div
              v-for="(column, index) in localColumns"
              :key="column.key"
              class="cell"
              :class="{ 'flex-grow': index === localColumns.length - 1 }"
              :style="column.isFlex ? {} : { width: `${column.width}px` }"
            >
              <!-- :get-popup-container="
                  (triggerNode) => triggerNode.parentNode as HTMLElement
                " -->
              <Tooltip
                v-if="shouldShowTooltip(item[column.key], column)"
                :destroyTooltipOnHide="false"
                :disabled="resizeState.isResizing"
                :mouseEnterDelay="0.3"
                placement="topLeft"
              >
                <template #title>
                  <div class="tooltip-content">
                    {{ getTooltipContent(item[column.key]) }}
                  </div>
                </template>
                <template #default>
                  <span
                    class="cell-content"
                    v-html="formatCellContent(item[column.key], column)"
                  />
                </template>
              </Tooltip>
              <span
                v-else
                class="cell-content"
                v-html="formatCellContent(item[column.key], column)"
              ></span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  onMounted,
  onUnmounted,
  reactive,
  watch,
  nextTick
} from "vue";
import { useTableLayoutStore } from "@/stores/table-layout";
import { Tooltip } from "ant-design-vue";
import { TrafficData, useTrafficStore } from "@/stores/traffic";

interface Column {
  key: string;
  title: string;
  width: number | "flex";
  minWidth?: number;
  originalWidth?: number | string;
  formatter?: (value: any) => string;
  isFlex?: boolean;
}

interface Props {
  data: any[];
  columns: Column[];
  rowHeight?: number;
  tableId: string;
  active: number | null;
}

const emit = defineEmits<{
  (e: "onCellClick", item: any): void;
  (e: "onContextMenu", event: any, traffic: TrafficData): void;
}>();

// 抛出点击事件
const onCellClick = (item: any) => {
  emit("onCellClick", item);
};

// 抛出右键事件
const onContextMenu = (e: MouseEvent, traffic: TrafficData) => {
  emit("onContextMenu", e, traffic);
};

const props = withDefaults(defineProps<Props>(), {
  rowHeight: 40
});

const tableLayoutStore = useTableLayoutStore();

// 在 script 部分添加以下内容

// 定义排序方向类型
type SortDirection = "asc" | "desc" | null;

// 排序状态接口
interface SortState {
  key: string | null;
  direction: SortDirection;
}

// 添加排序状态
const sortState = reactive<SortState>({
  key: null,
  direction: null
});

// 排序处理函数
const handleSort = (column: Column) => {
  // 如果是当前排序列，切换排序方向
  if (sortState.key === column.key) {
    if (sortState.direction === "asc") {
      sortState.direction = "desc";
    } else if (sortState.direction === "desc") {
      sortState.direction = null;
      sortState.key = null;
    }
  } else {
    // 如果是新的排序列，设置为升序
    sortState.key = column.key;
    sortState.direction = "asc";
  }
};

// 排序比较函数
const compareValues = (a: any, b: any, column: string): number => {
  const valueA = a[column];
  const valueB = b[column];

  // 根据不同列类型进行排序
  switch (column) {
    case "start_time":
    case "time":
      return new Date(valueA).getTime() - new Date(valueB).getTime();
    case "code":
    case "size":
      return Number(valueA) - Number(valueB);
    case "host":
    case "path":
    default:
      return String(valueA).localeCompare(String(valueB));
  }
};

// 修改 visibleData 计算属性
const visibleData = computed(() => {
  let sortedData = [...props.data];

  // 如果有排序状态，进行排序
  if (sortState.key && sortState.direction) {
    sortedData.sort((a, b) => {
      const compareResult = compareValues(a, b, sortState.key!);
      return sortState.direction === "asc" ? compareResult : -compareResult;
    });
  }

  const startIndex = Math.floor(scrollTop.value / props.rowHeight);
  const endIndex = Math.min(
    startIndex + Math.ceil(visibleHeight.value / props.rowHeight) + 10,
    sortedData.length
  );

  return sortedData.slice(startIndex, endIndex);
});

// 格式化单元格内容
const formatCellContent = (value: string, column: Column) => {
  return column.formatter ? column.formatter(value) : value;
};

// 是否显示 Tooltip
const shouldShowTooltip = (value: string, column: Column) => {
  // console.log(column);
  // 如果内容为空或很短，不显示 Tooltip
  if (!value) return false;

  if (resizeState.isResizing) return false;

  if (column.key === "transaction_state") return false;
  // console.log(column);
  const cellWidth =
    typeof column.width === "number" && column.width === 0 ? column.width : 128;

  let contentDom = document.getElementById("Measurer");

  if (!contentDom) {
    contentDom = document.createElement("div");
    contentDom.id = "Measurer";
    contentDom.style.cssText = `
        white-space: nowrap;
        position: absolute;
        left: -9999px;
        visibility: hidden;
      `;

    document.body.appendChild(contentDom);
  }
  contentDom.textContent =
    column.key === "start_time"
      ? formatCellContent(value, column)
      : (value ?? "");
  const contentWidth = contentDom.offsetWidth;

  return cellWidth < contentWidth;
};

// 获取 Tooltip 内容
const getTooltipContent = (value: string) => {
  const MAX_LENGTH = 600;

  // 转换为字符串并限制长度
  const content = String(value);
  if (content.length <= MAX_LENGTH) {
    return content;
  }

  // 超过长度限制时截断并添加省略号
  return content.slice(0, MAX_LENGTH) + ". . .";
};

// 初始化列配置，优先使用存储的配置
const initColumns = () => {
  // 首先尝试获取已存储的配置
  const storedColumns = tableLayoutStore.getColumnLayout(props.tableId);

  // 如果没有存储的配置，使用当前props的columns并保存
  if (storedColumns.length === 0) {
    const initialColumns = props.columns.map((col) => ({
      key: col.key,
      width: typeof col.width === "number" ? col.width : 10, // 默认10%
      minWidth: col.minWidth || 50
    }));

    // 保存初始配置
    tableLayoutStore.saveColumnLayout(props.tableId, initialColumns);
    return initialColumns;
  }

  return storedColumns;
};

// 父容器宽度
const parentWidth = ref(0);

// 计算列宽
const computedWidth = (columns: any[], parentWidth: number) => {
  return columns.map((col, index) => {
    // 如果是最后一列，标记为flex
    if (index === columns.length - 1) {
      return {
        ...col,
        isFlex: true,
        width: 0
      };
    }

    // 处理宽度计算
    let width = col.width;

    // 如果width是百分比
    if (typeof width === "number" && width <= 100) {
      width = parentWidth * (width / 100);
    }
    // 如果width是0或undefined，使用默认百分比
    else if (!width) {
      width = parentWidth * 0.1; // 默认10%
    }

    return {
      ...col,
      width: width,
      originalWidth: col.width, // 保留原始配置
      minWidth: col.minWidth || 50,
      isFlex: false
    };
  });
};

// 响应式列配置
const localColumns = ref(computedWidth(initColumns(), parentWidth.value));

// Refs
const containerRef = ref<HTMLDivElement | null>(null);
const scrollContainerRef = ref<HTMLDivElement | null>(null);

// 列宽调整状态
const resizeState = reactive({
  currentColumn: null as number | null,
  startX: 0,
  startWidth: 0,
  isResizing: false
});

// 开始调整列宽
const startResize = (e: MouseEvent, index: number) => {
  // 如果是最后一列或者是flex列，不允许调整
  if (
    index === localColumns.value.length - 1 ||
    localColumns.value[index].isFlex
  )
    return;

  // 检查是否点击在分隔线上
  const target = e.target as HTMLDivElement;
  if (!target.classList.contains("column-resizer")) return;

  e.preventDefault();

  resizeState.currentColumn = index;
  resizeState.startX = e.clientX;
  resizeState.startWidth = localColumns.value[index].width as number;
  resizeState.isResizing = true;

  // 添加鼠标移动和松开事件监听
  document.addEventListener("mousemove", onResize);
  document.addEventListener("mouseup", stopResize);
};

// 调整列宽
const onResize = (e: MouseEvent) => {
  if (resizeState.currentColumn === null) return;

  const deltaX = e.clientX - resizeState.startX;
  const column = localColumns.value[resizeState.currentColumn];

  // 计算新的列宽，确保不小于最小宽度
  const newWidth = Math.max(column.minWidth!, resizeState.startWidth + deltaX);

  // 更新列宽
  column.width = newWidth;

  // 保存到 store，传入父容器宽度
  tableLayoutStore.updateColumnWidth(
    props.tableId,
    column.key,
    newWidth,
    parentWidth.value
  );
};

// 结束调整
const stopResize = () => {
  resizeState.currentColumn = null;
  resizeState.isResizing = false;
  // 移除事件监听
  document.removeEventListener("mousemove", onResize);
  document.removeEventListener("mouseup", stopResize);
};

// 滚动状态
const scrollTop = ref(0);

// 计算总高度
const totalHeight = computed(() => {
  return props.data.length * props.rowHeight;
});

// 计算可视区域高度
const visibleHeight = ref(0);

// 计算可见数据
// const visibleData = computed(() => {
//   const startIndex = Math.floor(scrollTop.value / props.rowHeight);
//   const endIndex = Math.min(
//     startIndex + Math.ceil(visibleHeight.value / props.rowHeight) + 10,
//     props.data.length
//   );

//   return props.data.slice(startIndex, endIndex);
// });

// 计算起始偏移量
const startOffset = computed(() => {
  const startIndex = Math.floor(scrollTop.value / props.rowHeight);
  return startIndex * props.rowHeight;
});

// 滚动事件处理
const onScroll = (e: Event) => {
  const target = e.target as HTMLDivElement;
  scrollTop.value = target.scrollTop;
};

// 调整容器高度
const adjustContainerHeight = () => {
  const parentElement = containerRef.value?.parentElement;
  if (!parentElement) return;

  const rect = parentElement.getBoundingClientRect();
  parentWidth.value = rect.width;

  // 获取存储的列配置
  const storedColumns = tableLayoutStore.getColumnLayout(props.tableId);

  // 如果没有存储的配置，使用初始配置
  const columnsToCompute =
    storedColumns.length > 0
      ? storedColumns
      : props.columns.map((col) => ({
          key: col.key,
          width: typeof col.width === "number" ? col.width : 10, // 默认10%
          minWidth: col.minWidth || 50
        }));

  // 重新计算列宽
  localColumns.value = computedWidth(
    columnsToCompute.map((storedCol, index) => {
      const originalColumn = props.columns[index];
      return {
        ...originalColumn,
        width: storedCol.width // 使用存储的宽度或初始宽度
      };
    }),
    parentWidth.value
  );

  visibleHeight.value = Math.min(
    rect.height - 38,
    window.innerHeight - rect.top
  );

  if (scrollContainerRef.value) {
    scrollContainerRef.value.style.height = `${visibleHeight.value}px`;
  }
};

const scrollToBottom = () => {
  if (!scrollContainerRef.value) return;

  const scrollContainer = scrollContainerRef.value;
  scrollContainer.scrollTo({
    top: scrollContainer.scrollHeight,
    behavior: "smooth"
  });
};

const trafficStore = useTrafficStore();

watch(
  () => props.data,
  (newData, oldData) => {
    if (!trafficStore.isAutoScroll) return;

    // 只有在数据增加时才自动滚动
    if (newData.length > oldData.length) {
      // 使用 nextTick 确保 DOM 更新后再滚动
      nextTick(() => {
        scrollToBottom();
      });
    }
  }
);

// 监听 props.columns 变化
watch(
  () => props.columns,
  () => {
    adjustContainerHeight();
  },
  { deep: true }
);

// ResizeObserver
let resizeObserver: ResizeObserver | null = null;

// 初始化尺寸监听
const initResizeObserver = () => {
  if (!containerRef.value) return;

  resizeObserver = new ResizeObserver(() => {
    adjustContainerHeight();
  });

  resizeObserver.observe(containerRef.value.parentElement!);
};

// 生命周期钩子
onMounted(() => {
  adjustContainerHeight();
  initResizeObserver();
});

// 卸载清理
onUnmounted(() => {
  // 保存最终的列配置，确保使用百分比
  tableLayoutStore.saveColumnLayout(
    props.tableId,
    localColumns.value.map((col) => ({
      key: col.key,
      width: Number(((col.width / parentWidth.value) * 100).toFixed(2)),
      minWidth: col.minWidth
    }))
  );
});

// 对外暴露一些方法（可选）
defineExpose({
  adjustContainerHeight
});
</script>

<style scoped>
.virtual-table-container {
  width: 100%;
  position: relative;
  overflow: hidden;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.table-header {
  display: flex;
  background-color: #f8f9fa;
  border-bottom: 1px solid #e0e0e0;
  position: sticky;
  top: 0;
  z-index: 10;
  width: 100%;
}

.header-cell {
  padding: 12px 15px;
  font-weight: 600;
  color: #333;
  text-transform: uppercase;
  font-size: 12px;
  letter-spacing: 0.5px;
  border-right: 1px solid #e0e0e0;
  position: relative;
  user-select: none;
  transition: background-color 0.2s ease;
  /* flex: 1; */
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.header-cell:last-child {
  border-right: none;
}

.flex-grow {
  flex-grow: 1;
}

/* 列宽调整分隔线 */
.column-resizer {
  position: absolute;
  right: -5px;
  top: 0;
  bottom: 0;
  width: 10px;
  cursor: col-resize;
  z-index: 10;
  background-color: transparent;
}

.scroll-container {
  overflow-y: auto;
  overflow-x: hidden;
  width: 100%;
}

.scroll-height {
  position: relative;
}

.virtual-list {
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
}

.virtual-row {
  display: flex;
  align-items: center;
  border-bottom: 1px solid #e9ecef;
  transition:
    background-color 0.2s ease,
    transform 0.05s ease;
}

/* 选中状态的优先级要高于hover状态 */
.row-active {
  background-color: #e6f2ff;
  border-left: 3px solid #3498db;
}

/* 悬浮状态，但未选中 */
.virtual-row:hover:not(.row-active) {
  background-color: #f8f9fa;
  transform: translateX(5px);
}

/* 选中且悬浮状态 */
.row-active:hover {
  background-color: #daeeff;
  transform: translateX(5px);
}

.cell {
  padding: 12px 15px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: left;
  font-size: 14px;
  color: #495057;
  border-right: 1px solid #f1f3f5;
  /* flex: 1; */
  min-width: 0;
}

.cell:last-child {
  border-right: none;
}

/* 滚动条样式 */
.scroll-container::-webkit-scrollbar {
  width: 8px;
}

.scroll-container::-webkit-scrollbar-track {
  background: #f1f3f5;
  border-radius: 10px;
}

.scroll-container::-webkit-scrollbar-thumb {
  background: #adb5bd;
  border-radius: 10px;
}

.scroll-container::-webkit-scrollbar-thumb:hover {
  background: #868e96;
}

:deep(.ant-tooltip) {
  max-width: 95%;
}

:deep(.ant-tooltip-inner) {
  word-break: break-all;
  white-space: normal;
  display: block;
}

.header-cell {
  cursor: pointer;
}

.header-content {
  display: flex;
  align-items: center;
  gap: 4px;
}

.sort-icon {
  font-size: 12px;
  color: #666;
}

.sortable:hover {
  background-color: #eee;
}

.sorted {
  background-color: #f0f0f0;
}
</style>

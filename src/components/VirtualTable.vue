<template>
  <div
    tabindex="0"
    @keydown.prevent="onKeydown($event)"
    @keyup.prevent="onKeyup($event)"
    ref="containerRef"
    class="virtual-table-container"
  >
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
            :class="{
              'row-active': active === item.id,
              'row-active-select': ids && ids.has(item.id)
            }"
            @click.stop="onCellClick(item)"
            @dblclick="onDbLCellClick(item)"
            @click.right="onContextMenu($event, item)"
            @mousedown="onMouseDown($event, item)"
            @mouseup="onMouseUp($event, item)"
            @mouseenter="onMouseEnter($event, item)"
          >
            <!-- :title="getTooltipContent(String(item[column.key]))" -->
            <div
              :title="String(item[column.key])"
              v-for="(column, index) in localColumns"
              :key="column.key + '-' + index + '-cell' + item.id"
              class="cell"
              :class="{ 'flex-grow': index === localColumns.length - 1 }"
              :style="column.isFlex ? {} : { width: `${column.width}px` }"
            >
              <!-- :get-popup-container="
                  (triggerNode) => triggerNode.parentNode as HTMLElement
                " -->
              <!-- <Tooltip
                v-if="shouldShowTooltip(String(item[column.key]), column)"
                :destroyTooltipOnHide="false"
                :disabled="resizeState.isResizing"
                :mouseEnterDelay="0.3"
                placement="topLeft"
              >
                <template #title>
                  <div
                    v-html="getTooltipContent(String(item[column.key]))"
                    class="tooltip-content"
                  />
                </template>
                <template #default>
                  <span
                    class="cell-content"
                    v-html="formatCellContent(String(item[column.key]), column)"
                  />
                </template>
              </Tooltip> -->
              <!-- v-else -->
              <span
                class="cell-content"
                v-html="formatCellContent(String(item[column.key]), column)"
              ></span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive, watch, nextTick } from "vue";
import { type TrafficData, useTrafficStore } from "@/stores/traffic";
import { useSessionStore } from "@/stores/session";

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
const props = withDefaults(defineProps<Props>(), {
  rowHeight: 40
});

const ids = defineModel<Set<number>>("ids", {
  required: false
});

const emit = defineEmits<{
  onCellClick: [item: TrafficData];
  onDbLCellClick: [item: TrafficData];
  onContextMenu: [event: any, traffic: TrafficData];
  onCellMouseDown: [event: MouseEvent, traffic: TrafficData];
  onCellMouseUp: [event: MouseEvent, traffic: TrafficData];
  onCellMouseEnter: [event: MouseEvent, traffic: TrafficData];
  onCellKeydown: [event: KeyboardEvent];
  onCellKeyup: [event: KeyboardEvent];
}>();
// 处理按下拖动多选
const multiSelect = ref<{
  startId: number;
  endId: number;
  isMouseSelecting: boolean;
  isShiftSelecting: boolean;
  isCtrlSelecting: boolean;
}>({
  startId: -1,
  endId: -1,
  isMouseSelecting: false,
  isShiftSelecting: false,
  isCtrlSelecting: false
});

// 抛出点击事件
const onCellClick = (traffic: TrafficData) => {
  emit("onCellClick", traffic);
};

// 抛出双击事件
const onDbLCellClick = (traffic: TrafficData) => {
  emit("onDbLCellClick", traffic);
};

const sessionStore = useSessionStore();

watch(
  () => sessionStore.currentSession,
  () => {
    ids.value = new Set();
  }
);

const getTrafficsBetweenIds = (startId: number, endId: number) => {
  if (!sessionStore.currentSession) return [];
  const list = trafficStore.trafficList.get(sessionStore.currentSession);
  if (!list) return [];
  // 将 Map 转换为数组
  const trafficList = Array.from(list.values());

  const startIndex = trafficList.findIndex((t) => t.id === startId);
  const endIndex = trafficList.findIndex((t) => t.id === endId);

  // 如果找不到，返回空
  if (startIndex === -1 || endIndex === -1) return [];

  // 确定正确的起始和结束索引
  const actualStart = Math.min(startIndex, endIndex);
  const actualEnd = Math.max(startIndex, endIndex);

  // 返回这个区间的流量
  return trafficList.slice(actualStart, actualEnd + 1);
};

let fistId = -1;

// 鼠标按下事件
const onMouseDown = (e: MouseEvent, traffic: TrafficData) => {
  emit("onCellMouseDown", e, traffic);
  if (!ids.value) return;
  multiSelect.value.startId = fistId;
  if (multiSelect.value.isCtrlSelecting) {
    if (ids.value.has(traffic.id)) {
      ids.value.delete(traffic.id);
      return;
    }

    ids.value.add(traffic.id);
    return;
  } else {
    if (e.button === 2) return;
    ids.value.clear();
  }

  if (multiSelect.value.isShiftSelecting) {
    multiSelect.value.endId = traffic.id;

    if (!multiSelect.value.startId || !multiSelect.value.endId) return;

    const trafficsBetween = getTrafficsBetweenIds(
      multiSelect.value.startId,
      traffic.id
    );
    trafficsBetween.forEach((t) => ids.value?.add(t.id));
    return;
  }
  multiSelect.value.isMouseSelecting = true;
  fistId = traffic.id;
};

// 抛出鼠标抬起事件
const onMouseUp = (e: MouseEvent, traffic: TrafficData) => {
  emit("onCellMouseUp", e, traffic);
  if (!ids.value) return;
  multiSelect.value.isMouseSelecting = false;
};

// 键盘按下事件
const onKeydown = (e: KeyboardEvent) => {
  emit("onCellKeydown", e);
  if (!ids.value) return;
  if (multiSelect.value.isShiftSelecting || multiSelect.value.isCtrlSelecting)
    return;
  e.preventDefault();
  if (e.shiftKey) {
    if (multiSelect.value.isShiftSelecting) return;

    multiSelect.value.isShiftSelecting = true;
  }
  if (e.ctrlKey) {
    if (multiSelect.value.isCtrlSelecting) return;

    multiSelect.value.isCtrlSelecting = true;
  }
};

// 键盘抬起事件
const onKeyup = (e: KeyboardEvent) => {
  emit("onCellKeyup", e);
  e.preventDefault();
  if (!ids.value) return;
  if (e.key === "Shift") {
    if (!multiSelect.value.isShiftSelecting) return;

    multiSelect.value.isShiftSelecting = false;
  }
  if (e.key === "Control") {
    if (!multiSelect.value.isCtrlSelecting) return;

    multiSelect.value.isCtrlSelecting = false;
  }
};

// 抛出右键事件
const onContextMenu = (e: MouseEvent, traffic: TrafficData) => {
  emit("onContextMenu", e, traffic);
};

let lastId = -1;
// 处理拖动多选
const handleMultiSelect = (traffic: TrafficData) => {
  if (!ids.value) return;
  if (!multiSelect.value.isMouseSelecting) return;

  // 处理第一次拖动
  if (!ids.value.has(fistId)) {
    ids.value.add(fistId);
  }

  // 处理拖动后往回拖
  if (ids.value.has(traffic.id)) {
    if (ids.value.has(lastId)) {
      ids.value.delete(lastId);
    }
    ids.value.delete(traffic.id);
    return;
  }

  // 正常选择
  ids.value.add(traffic.id);
  lastId = traffic.id;
};

// 鼠标进入事件
const onMouseEnter = (e: MouseEvent, traffic: TrafficData) => {
  emit("onCellMouseEnter", e, traffic);
  if (!ids.value) return;
  if (!multiSelect.value.isMouseSelecting) return;
  handleMultiSelect(traffic);
};

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
const visibleData = computed<TrafficData[]>(() => {
  const sortedData = [...props.data];

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
const formatCellContent = (value: string | null, column: Column) => {
  return column.formatter ? column.formatter(value) : value;
};

// 是否显示 Tooltip
// const shouldShowTooltip = (value: string | null, column: Column) => {
//   // 如果内容为空或很短，不显示 Tooltip
//   if (!value) return false;

//   if (resizeState.isResizing) return false;
//   const cellWidth =
//     typeof column.width === "number" && column.width === 0 ? column.width : 128;

//   let contentDom = document.getElementById("Measurer");

//   if (!contentDom) {
//     contentDom = document.createElement("div");
//     contentDom.id = "Measurer";
//     contentDom.style.cssText = `
//         white-space: nowrap;
//         position: absolute;
//         left: -9999px;
//         visibility: hidden;
//       `;

//     document.body.appendChild(contentDom);
//   }
//   contentDom.textContent =
//     column.key === "start_time"
//       ? formatCellContent(value, column)
//       : (value ?? "");
//   const contentWidth = contentDom.offsetWidth;

//   return cellWidth < contentWidth;
// };

// 获取 Tooltip 内容
// const getTooltipContent = (value: string | null) => {
//   const MAX_LENGTH = 600;
//   // 如果内容为空，返回空字符串
//   if (!value) return "";

//   // 转换为字符串并限制长度
//   const content = String(value);
//   if (content.length <= MAX_LENGTH) {
//     return content;
//   }

//   // 超过长度限制时截断并添加省略号
//   return content.slice(0, MAX_LENGTH) + ". . .";
// };

// 父容器宽度
const parentWidth = ref(0);

// 计算列宽
const computedWidth = (columns: Column[], parentWidth: number) => {
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
const localColumns = ref<Column[]>(
  computedWidth(props.columns, parentWidth.value)
);

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

  // 如果没有存储的配置，使用初始配置
  const columnsToCompute = props.columns.map((col) => ({
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

const scrollToId = (id: number) => {
  // 首先找到对应 ID 的数据项在整个数据集中的索引
  const index = props.data.findIndex((item) => item.id === id);
  if (index === -1) return; // 如果没找到，直接返回

  // 计算目标行的垂直位置
  const targetScrollTop = index * props.rowHeight;

  // 使用 nextTick 确保 DOM 已更新
  nextTick(() => {
    if (!scrollContainerRef.value) return;

    // 平滑滚动到目标位置
    scrollContainerRef.value.scrollTo({
      top: targetScrollTop - visibleHeight.value / 2 + props.rowHeight / 2,
      behavior: "smooth"
    });
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
// 对外暴露一些方法（可选）
defineExpose({
  adjustContainerHeight,
  scrollToId
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
.virtual-table-container * {
  user-select: none;
}

.virtual-table-container:focus {
  outline: none; /* 移除默认轮廓 */
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2); /* 自定义焦点指示 */
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
}

/* 选中状态的优先级要高于hover状态 */
.row-active {
  background-color: #b7d0ea;
  border-left: 3px solid #0a5383;
}

.row-active-select {
  background-color: #b7baea;
  border-left: 3px solid #0a2883;
}

/* 悬浮状态，但未选中 */
.virtual-row:hover:not(.row-active):not(.row-active-select) {
  background-color: #daedff;
}

/* 选中且悬浮状态 */
.row-active:hover {
  background-color: #b7d0ea;
}

.cell {
  padding: 12px 15px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: left;
  font-size: 14px;
  color: #495057;
  /* border-right: 1px solid #f1f3f5; */
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

<template>
  <div ref="tableContainer" class="table-container">
    <el-table
      style="width: 100%"
      ref="tableRef"
      v-bind="tableOps"
      :data="dataSource"
      show-overflow-tooltip
      @row-dblclick="handleRowDblClick"
      @row-contextmenu="handleRowContextmenu"
      :tooltip-formatter="onTooltipFormatter"
    >
      <!-- Name 列 -->
      <el-table-column
        prop="name"
        label="Name"
        width="100"
        min-width="100"
        fixed="left"
      >
        <template #default="{ row, $index }">
          <template v-if="editingIndex === $index">
            <el-input
              v-model="row.name"
              size="small"
              @keyup.enter="saveEdit()"
              @blur="saveEdit()"
            />
          </template>
          <span v-else class="table-cell-ellipsis">
            {{ row.name }}
          </span>
        </template>
      </el-table-column>

      <!-- Value 列 -->
      <!-- width="100" -->
      <el-table-column
        min-width="100"
        :resizable="false"
        prop="value"
        label="Value"
      >
        <!-- show-overflow-tooltip -->
        <template #default="{ row, $index }">
          <template v-if="editingIndex === $index">
            <el-input
              v-model="row.value"
              size="small"
              @keyup.enter="saveEdit()"
              @blur="saveEdit()"
            />
          </template>
          <span v-else class="table-cell-ellipsis">
            {{ row.value }}
          </span>
        </template>
      </el-table-column>

      <!-- 操作列 -->
      <el-table-column
        :resizable="false"
        fixed="right"
        v-if="!readOnly"
        label="Operation"
        width="140"
      >
        <template #default="{ $index }">
          <template v-if="editingIndex === $index">
            <el-button-group>
              <el-button type="primary" size="small" @click="saveEdit">
                Save
              </el-button>
              <el-button size="small" @click="cancelEdit"> Cancel </el-button>
            </el-button-group>
          </template>
          <template v-else>
            <el-button-group>
              <el-button size="small" @click="startEdit($index)">
                Edit
              </el-button>
              <el-button
                size="small"
                type="danger"
                @click="deleteRecord($index)"
              >
                Delete
              </el-button>
            </el-button-group>
          </template>
        </template>
      </el-table-column>
    </el-table>

    <footer v-if="!readOnly" class="table-footer">
      <el-button type="primary" size="small" @click="addRecord">
        Add
      </el-button>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { h, nextTick, ref } from "vue";
import {
  ElTable,
  ElTableColumn,
  ElInput,
  ElButton,
  ElButtonGroup
} from "element-plus";
import { tableOps, type DataItem } from "@/window/breakpoint/pause/edit/model";
import ContextMenu from "@imengyu/vue3-context-menu";
import { IHeaderItem } from "@/stores/traffic";

defineOptions({ name: "EditHeader" });

// 双向绑定 dataSource
const dataSource = defineModel<DataItem[]>("dataSource", {
  required: true
});

const { readOnly = false } = defineProps<{
  readOnly?: boolean;
}>();

// 表格和编辑状态
const tableRef = ref<InstanceType<typeof ElTable>>();
const editingIndex = ref(-1);
const originalRecord = ref<Partial<DataItem> | null>(null);

// 处理行双击事件
const handleRowDblClick = (row: DataItem) => {
  if (readOnly) return;
  const index = dataSource.value.indexOf(row);
  startEdit(index);
};

// 处理行右键事件
const handleRowContextmenu = (row: DataItem, _: any, event: MouseEvent) => {
  if (readOnly) return;

  event.preventDefault();
  const index = dataSource.value.indexOf(row);

  const getContextMenuItems = () => {
    return editingIndex.value === index
      ? [
          {
            label: "保存",
            onClick: () => saveEdit()
          },
          {
            label: "取消",
            onClick: cancelEdit
          }
        ]
      : [
          {
            label: "编辑",
            onClick: () => startEdit(index)
          },
          {
            label: "删除",
            onClick: () => deleteRecord(index)
          }
        ];
  };

  ContextMenu.showContextMenu({
    x: event.x,
    y: event.y,
    items: getContextMenuItems()
  });
};

// 添加记录
const addRecord = () => {
  const newRecord: DataItem = {
    name: "New Name",
    value: "New Value"
  };

  dataSource.value = [...dataSource.value, newRecord];

  nextTick(() => {
    tableRef.value?.scrollTo({
      top: Number.MAX_SAFE_INTEGER,
      behavior: "smooth"
    });

    const lastIndex = dataSource.value.length - 1;
    startEdit(lastIndex);
  });
};

// 开始编辑
const startEdit = (index: number) => {
  if (editingIndex.value !== -1) {
    cancelEdit();
  }

  originalRecord.value = { ...dataSource.value[index] };
  editingIndex.value = index;
};

// 保存编辑
const saveEdit = () => {
  editingIndex.value = -1;
  originalRecord.value = null;
};

// 取消编辑
const cancelEdit = () => {
  if (originalRecord.value !== null && editingIndex.value !== -1) {
    dataSource.value[editingIndex.value] = {
      ...(originalRecord.value as DataItem)
    };
    editingIndex.value = -1;
    originalRecord.value = null;
  }
};

// 删除记录
const deleteRecord = (index: number) => {
  dataSource.value.splice(index, 1);
};

const onTooltipFormatter = ({ row }: { row: IHeaderItem }) => {
  return h(
    "div",
    {
      style: {
        display: "inline-block",
        maxWidth: "90vw",
        overflow: "hidden"
      }
    },
    row.value
  );
};
</script>

<style scoped>
.table-container {
  width: 100%;
  height: 100%;
  /* height: calc(100% - 32px); */
  display: flex;
  flex-direction: column;
}

.table-footer {
  width: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  /* padding: 10px; */
  height: 28px;
}

.table-cell-ellipsis {
  display: inline-block;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>

import { defineStore } from "pinia";
import { ref } from "vue";

interface ColumnConfig {
  key: string;
  width: number | string;
  minWidth?: number;
  originalWidth?: number | string; // 新增原始宽度配置
}

export const useTableLayoutStore = defineStore("tableLayout", () => {
  const columnConfigs = ref<Record<string, ColumnConfig[]>>({});

  // 保存列布局，增加原始宽度保存
  const saveColumnLayout = (tableId: string, columns: ColumnConfig[]) => {
    columnConfigs.value[tableId] = columns.map((col) => ({
      key: col.key,
      width: col.width,
      minWidth: col.minWidth,
      originalWidth: col.originalWidth || col.width, // 保存原始宽度配置
    }));
  };

  // 获取列布局
  const getColumnLayout = (tableId: string): ColumnConfig[] => {
    return columnConfigs.value[tableId] || [];
  };

  // 更新单个列宽度
  const updateColumnWidth = (tableId: string, key: string, width: number, parentWidth: number) => {
    const tableColumns = columnConfigs.value[tableId] || [];
    const columnIndex = tableColumns.findIndex((col) => col.key === key);

    if (columnIndex !== -1) {
      // 将像素宽度转换为百分比
      const widthPercentage = (width / parentWidth) * 100;

      tableColumns[columnIndex].width = Number(widthPercentage.toFixed(2));
      columnConfigs.value[tableId] = [...tableColumns];
    }
  };

  // 重置特定表格的列配置
  const resetColumnLayout = (tableId: string) => {
    if (columnConfigs.value[tableId]) {
      columnConfigs.value[tableId] = columnConfigs.value[tableId].map((col) => ({
        ...col,
        width: col.originalWidth || col.width,
      }));
    }
  };

  // 清除所有表格配置
  const clearAllLayouts = () => {
    columnConfigs.value = {};
  };

  return {
    columnConfigs,
    saveColumnLayout,
    getColumnLayout,
    updateColumnWidth,
    resetColumnLayout,
    clearAllLayouts,
  };
});

<template>
  <ElTable class="h-100%" border :data="dataSource">
    <ElTableColumn prop="name" label="name" width="150">
      <template #default="scope">
        <b>{{ toPascalCase(scope.row.name) }}</b>
      </template>
    </ElTableColumn>
    <ElTableColumn prop="value" label="value" />
    <template #empty>
      <div></div>
    </template>
  </ElTable>
</template>

<script setup lang="ts">
import type { Overview } from "@/stores/traffic";
import type { DataItem } from "@/components/contents/model";
import { ElTable, ElTableColumn } from "element-plus";
import { computed } from "vue";

function toPascalCase(snakeStr: string) {
  return snakeStr
    .split("_") // 分割字符串
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase()) // 首字母大写，其他字母小写
    .join(" "); // 连接成字符串
}

const { overview } = defineProps<{
  overview: Partial<Overview>;
}>();

const dataSource = computed(() => {
  const data: DataItem[] = [];
  for (const key in overview) {
    data.push({
      name: key,
      value: overview[key]
    });
  }
  return data;
});
</script>

<style scoped></style>

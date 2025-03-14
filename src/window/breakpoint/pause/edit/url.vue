<template>
  <div class="w f-col-start-center h-100%">
    <header class="w header p-4px">
      <div class="w-100% f-l f-g-10">
        <Select size="small" class="w-150px" v-model:value="urlEditData.method">
          <Select.Option
            v-for="op in urlSelectOps"
            :key="op.value"
            :value="op.value"
          >
            {{ op.label }}
          </Select.Option>
        </Select>

        <div class="w">
          <Input size="small" v-model:value="urlEditData.url" />
        </div>

        <Select
          size="small"
          v-model:value="urlEditData.httpVersion"
          class="w-150px"
        >
          <Select.Option
            v-for="op in httpVersionOps"
            :key="op.value"
            :value="op.value"
          >
            {{ op.label }}
          </Select.Option>
        </Select>
      </div>
    </header>

    <!--  -->
    <EditTable
      style="height: calc(100% - 32px)"
      v-model:data-source="dataSource"
    />
  </div>
</template>

<script setup lang="ts">
import { Select, Input } from "ant-design-vue";
import { urlSelectOps, urlData, httpVersionOps, DataItem } from "./model";
import EditTable from "@/components/EditTable.vue";

// URL 编辑数据
const urlEditData = defineModel<urlData>("urlEditData", {
  required: true
});
// Computed dataSource for two-way binding
const dataSource = defineModel<DataItem[] & Record<string, string>>(
  "dataSource",
  {
    required: true
  }
);
</script>

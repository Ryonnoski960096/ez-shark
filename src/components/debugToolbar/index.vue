<template>
  <div class="f-c editor-toolbar f-g-10">
    <Button type="primary" size="small" @click="isOpen = !isOpen">DEBUG</Button>
    <div class="f-c f-g-5" v-if="isOpen">
      <Button size="small" @click="clearStore" type="primary" danger
        >清除Store</Button
      >
      <Button size="small" type="primary" @click="getAllTraffics"
        >获取所有流量</Button
      >
    </div>
  </div>
</template>

<script setup lang="ts">
import { getTraffics } from "@/api/debug";
import { useSettingStore } from "@/stores/settings";
import { Button } from "ant-design-vue";
import { ref } from "vue";

const isOpen = ref(false);

async function getAllTraffics() {
  const res = await getTraffics();
  console.log(res);
}

defineOptions({
  name: "DebugToolbar"
});

function clearStore() {
  useSettingStore().clear();
}
</script>

<style scoped></style>

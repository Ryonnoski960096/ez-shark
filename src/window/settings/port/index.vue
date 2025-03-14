<template>
  <Page>
    <form @submit="submit" class="f-col-between-center f-g-20 form">
      <div class="f-l w">
        <label class="w-100px">端口： </label>
        <Input type="number" size="small" v-model:value="portData.port" />
      </div>

      <Button type="primary" size="small" htmlType="submit"> 确定 </Button>
    </form>
  </Page>
</template>

<script setup lang="ts">
import Page from "@/components/Page.vue";
import { PortEvent } from "@/enum/port";
import { windowInit, windowManager } from "@/stores/WindowManager";
import { useSettingStore } from "@/stores/settings";
import { message } from "@tauri-apps/plugin-dialog";
import { Input, Button } from "ant-design-vue";
import { ref } from "vue";

windowInit();

const settingStore = useSettingStore();

console.log("settingStore.settings.port", settingStore.settings.port);

const portData = ref({
  port: settingStore.settings.port ?? 8081
  // port: 9210
});

const submit = async (event: Event) => {
  event.preventDefault();
  const port = Number(portData.value.port);

  if (port < 1024 || port > 65535) {
    await message("端口号必须在1024到65535之间", { kind: "error" });
    return;
  }

  windowManager.window.emit(PortEvent.SUBMIT, port);
};
</script>

<style scoped>
.form {
  height: calc(100vh - 70px);
}
</style>

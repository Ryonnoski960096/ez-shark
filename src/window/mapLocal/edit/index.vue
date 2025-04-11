<template>
  <Page>
    <Form
      size="small"
      layout="horizontal"
      :model="mapLocalItem"
      class="w"
      @submit="onSubmit($event)"
    >
      <div class="f-col-center-center">
        <table style="border-spacing: 0 10px">
          <tbody>
            <tr>
              <td style="padding: 5px">URL:</td>
              <td class="w" style="padding: 5px">
                <Input v-model:value="mapLocalItem.url" />
              </td>
            </tr>
            <tr>
              <td style="padding: 5px">HeaderLocal:</td>
              <td class="w" style="padding: 5px">
                <div class="flex f-g-5">
                  <Button @click="selectHeaderFile">选择文件</Button>
                  <Input v-model:value="mapLocalItem.headerLocal" />
                </div>
              </td>
            </tr>
            <tr>
              <td style="padding: 5px">BodyLocal:</td>
              <td class="w" style="padding: 5px">
                <div class="flex f-g-5">
                  <Button @click="selectBodyFile">选择文件</Button>
                  <Input v-model:value="mapLocalItem.bodyLocal" />
                </div>
              </td>
            </tr>
          </tbody>
        </table>
        <Space class="mt-5px">
          <Button html-type="submit" type="primary"> 提交 </Button>
          <Button @click="handleCancel"> 取消 </Button>
        </Space>
      </div>
    </Form>
  </Page>
</template>

<script lang="tsx" setup>
import Page from "@/components/Page.vue";
import { Button, Form, Input, message, Space } from "ant-design-vue";
import { ref } from "vue";
import { MapLocalEvent, type MapLocalItem } from "../model";
import { windowInit, windowManager } from "@/stores/WindowManager";
import { open } from "@tauri-apps/plugin-dialog";
import { deepClone } from "@/utils/tools";

interface ExtendedMapLocalItem {
  url?: string;
  headerLocal?: string;
  bodyLocal?: string;
  parentWindowId?: string;
  enabled?: "true" | "false" | boolean;
  id?: string;
}

const prams = windowInit() as ExtendedMapLocalItem;
delete prams.parentWindowId;
const defaultMapLocalItem: MapLocalItem = {
  url: "",
  headerLocal: "",
  bodyLocal: "",
  enabled: true
};

const mapLocalItem = ref<MapLocalItem>(deepClone(defaultMapLocalItem));
if (prams) {
  console.log("prams", prams);
  if (prams.enabled === "true") {
    mapLocalItem.value.enabled = true;
  }
  if (prams.enabled === "false") {
    mapLocalItem.value.enabled = false;
  }
  mapLocalItem.value.bodyLocal = prams.bodyLocal ?? "";
  mapLocalItem.value.headerLocal = prams.headerLocal ?? "";
  mapLocalItem.value.url = prams.url ?? "";
  mapLocalItem.value.id = prams.id;
}

const selectHeaderFile = async () => {
  const path = await open({
    filters: [
      {
        name: "json",
        extensions: ["json"]
      }
    ]
  });
  if (!path) throw new Error("未选择文件");
  mapLocalItem.value.headerLocal = path;
};

const selectBodyFile = async () => {
  const path = await open();
  if (!path) throw new Error("未选择文件");
  mapLocalItem.value.bodyLocal = path;
};

const handleCancel = async () => {
  mapLocalItem.value = deepClone(defaultMapLocalItem);
  await windowManager.requestClose();
};

const onSubmit = async (e: Event) => {
  e.preventDefault();
  if (!mapLocalItem.value.url) {
    message.warning("请填写 URL");
    return;
  }

  // url 必填，headerLocal 和 bodyLocal 至少有一个
  if (!mapLocalItem.value.headerLocal && !mapLocalItem.value.bodyLocal) {
    message.warning("请填写 Header 或 Body 本地文件");
    return;
  }

  await windowManager.window.emit(MapLocalEvent.SUBMIT, mapLocalItem.value);
  handleCancel();
};
</script>

<style scoped></style>

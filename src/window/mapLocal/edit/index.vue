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

windowInit();

const defaultMapLocalItem: Omit<MapLocalItem, "id" | "enabled"> = {
  url: "",
  headerLocal: "",
  bodyLocal: ""
};

const mapLocalItem = ref<Omit<MapLocalItem, "id" | "enabled">>(
  deepClone(defaultMapLocalItem)
);

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
  if (
    !mapLocalItem.value.url ||
    !mapLocalItem.value.headerLocal ||
    !mapLocalItem.value.bodyLocal
  ) {
    message.warning("请填写完整信息");
    return;
  }

  await windowManager.window.emit(MapLocalEvent.SUBMIT, mapLocalItem.value);
  handleCancel();
};
</script>

<style scoped></style>

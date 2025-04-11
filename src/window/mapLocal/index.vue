<template>
  <Page>
    <p>使用本地文件替换响应数据</p>
    <p class="f-l f-g-10">
      <span>本地映射功能:</span>
      <Switch
        size="small"
        v-model:checked="mapLocal.toolEnabled"
        checked-children="开"
        un-checked-children="关"
      />
    </p>
    <div class="mapLocalList">
      <table>
        <thead>
          <tr>
            <th style="width: 34px">
              <Checkbox
                v-model:checked="allChecked"
                @change="handleAllChecked"
              />
            </th>
            <th style="width: 48px">
              <Switch
                size="small"
                @change="handleAllSwitch"
                v-model:checked="allSwitch"
              />
            </th>
            <th style="width: 150px">Location</th>
            <th>HeaderLocal</th>
            <th>BodyLocal</th>
          </tr>
        </thead>
        <tbody v-if="Object.keys(mapLocal?.mapLocals || {}).length">
          <tr
            @click.right="oncontextmenu($event, item, key)"
            v-for="(item, key, index) in mapLocal?.mapLocals"
            :key="key"
          >
            <td>
              <Checkbox v-model:checked="checkboxArr[index]" />
            </td>
            <td>
              <!-- @change="mapLocalsEnabled(item)" -->
              <Switch size="small" v-model:checked="item.enabled" />
            </td>
            <td style="width: 40%">
              {{ item.url }}
            </td>
            <td :title="item.headerLocal" style="width: 20%">
              {{ item.headerLocal }}
            </td>
            <td :title="item.bodyLocal" style="width: 20%">
              {{ item.bodyLocal }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <div style="margin-top: 20px" class="operation f-c f-g-20">
      <div class="f-c f-g-20">
        <Button @click="addMapLocal" size="small">添加</Button>
      </div>
      <div class="f-c f-g-20">
        <Button @click="removeMapLocalList" size="small" danger>删除</Button>
      </div>
    </div>

    <div>
      <div class="f-b">
        <div class="f-c f-g-10">
          <Button type="default" @click="importHandler">导入</Button>
          <Button type="default" @click="exportHandler">导出</Button>
        </div>
        <div class="f-c f-g-10">
          <Button key="back" @click="handleCancel">取消</Button>
          <Button key="submit" type="primary" @click="handleOk">完成</Button>
        </div>
      </div>
    </div>
  </Page>
</template>

<script setup lang="tsx">
import Page from "@/components/Page.vue";
import { Button, Checkbox, message, Switch } from "ant-design-vue";
import type { Reactive } from "vue";
import { onBeforeMount, reactive, ref, watch } from "vue";
import { MapLocalEvent, type MapLocal, type MapLocalItem } from "./model";
import { deepClone } from "@/utils/tools";
import { useSettingStore } from "@/stores/settings";
import ContextMenu from "@imengyu/vue3-context-menu";
import { windowInit, windowManager } from "@/stores/WindowManager";
import { confirm, open, save } from "@tauri-apps/plugin-dialog";
import { error } from "@tauri-apps/plugin-log";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { createXmlStr, readXmlFile } from "@/utils/file";

// 窗口初始化
windowInit();

const WINDOW_CONFIG = {
  WIDTH: 550,
  HEIGHT: 220,
  TITLE: "Map Local编辑器"
};

const defaultMapLocal: MapLocal = {
  toolEnabled: false,
  mapLocals: {}
};

const mapLocal = ref<MapLocal>(deepClone(defaultMapLocal));

const allSwitch = ref(false);
const allChecked = ref(false);

const newIds = ref<string[]>([]);

const settingStore = useSettingStore();

// 全选功能
const handleAllSwitch = () => {
  if (!mapLocal.value?.mapLocals) return;

  const isCurrentlyAllChecked = allSwitch.value;
  Object.keys(mapLocal.value.mapLocals).forEach((key) => {
    mapLocal.value!.mapLocals[key].enabled = isCurrentlyAllChecked;
  });

  settingStore.set("mapLocal", mapLocal.value);
};

const createFalseArray = () => {
  return reactive(
    new Proxy([], {
      get(target, prop) {
        // 优先返回原始属性
        if (prop in target || typeof prop === "symbol") {
          return target[prop as any];
        }

        // 数字索引返回 false
        return !isNaN(Number(prop)) ? false : undefined;
      }
    })
  );
};

const checkboxArr: Reactive<boolean[]> = createFalseArray();

const handleAllChecked = () => {
  if (!mapLocal.value?.mapLocals) return;
  for (let i = 0; i < checkboxArr.length; i++) {
    checkboxArr[i] = allChecked.value;
  }
};
const removeIds = ref<string[]>([]);

// 删除一个
const deleteMapLocal = async (key: string) => {
  if (!mapLocal.value) return;

  delete mapLocal.value.mapLocals[key];
  removeIds.value.push(key);
};

// 编辑
const editMapLocal = async (key: string) => {
  openWindow({ key });
};

function oncontextmenu(e: MouseEvent, _: MapLocalItem, key: string) {
  e.preventDefault();

  ContextMenu.showContextMenu({
    x: e.x,
    y: e.y,
    items: [
      {
        label: "编辑",
        onClick: () => editMapLocal(key)
      },
      {
        label: "删除",
        onClick: () => deleteMapLocal(key)
      }
    ]
  });
}

// 关闭前执行检查
async function handleCloseExamine() {
  if (newIds.value.length !== 0) {
    const res = await confirm("当前有未保存的配置，是否确认关闭？", {
      kind: "warning",
      okLabel: "确认",
      cancelLabel: "取消"
    });
    if (res) {
      for (const key in mapLocal.value!.mapLocals) {
        if (newIds.value.includes(key)) {
          delete mapLocal.value!.mapLocals[key];
        }
      }
      await settingStore.set("mapLocal", mapLocal.value);
      windowManager.closeTasks.delete(handleCloseExamine);
      return true;
    } else {
      return false;
    }
  }
  windowManager.closeTasks.delete(handleCloseExamine);
  return true;
}
windowManager.closeTasks.add(handleCloseExamine);
// 点击取消
const handleCancel = async () => {
  await windowManager.requestClose();
};

const handleOk = async () => {
  await settingStore.set("mapLocal", mapLocal.value);
  windowManager.closeTasks.delete(handleCloseExamine);

  handleCancel();
};

const generateTimestampKey = (): string => {
  return `mapLocal_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
};
const createMapLocal = (
  mapLocalItem: Omit<MapLocalItem, "enabled">
): [string, MapLocalItem] => {
  const id = mapLocalItem.id || generateTimestampKey();
  return [
    id,
    {
      id,
      enabled: true,
      ...mapLocalItem
    }
  ];
};

const openWindow = async (ops?: Record<string, any>) => {
  const [wvw] = await windowManager.createWindow(
    {
      url: "/mapLocal/edit",
      param: ops
    },
    {
      width: WINDOW_CONFIG.WIDTH,
      height: WINDOW_CONFIG.HEIGHT,
      title: WINDOW_CONFIG.TITLE
    }
  );

  // 监听事件
  const unListen = await wvw.listen(MapLocalEvent.SUBMIT, (event) => {
    try {
      const payload = event.payload as Omit<MapLocalItem, "id" | "enabled">;

      if (!mapLocal.value) {
        mapLocal.value = {
          toolEnabled: false,
          mapLocals: {}
        };
      }

      const [key, mapLocalItem] = createMapLocal(payload);
      mapLocal.value.mapLocals[key] = mapLocalItem;

      settingStore.set("mapLocal", mapLocal.value);

      newIds.value.push(key);
      message.success("添加成功");
    } catch (e) {
      error(e + "");
      message.error(`添加失败：${e instanceof Error ? e.message : "未知错误"}`);
    } finally {
      unListen();
    }
  });
};

const addMapLocal = () => {
  openWindow();
};

const removeMapLocalList = async () => {
  if (!mapLocal.value) return;

  const keys = Object.keys(mapLocal.value.mapLocals);
  for (let i = 0; i < checkboxArr.length; i++) {
    if (checkboxArr[i]) {
      removeIds.value.push(keys[i]);
      checkboxArr[i] = false;
    }
  }
  removeIds.value.forEach((key) => {
    delete mapLocal.value?.mapLocals[key];
  });
};
const state = reactive({
  mapLocal: false
});
const getTableData = async () => {
  const mapLocalData: MapLocal | null | undefined =
    await settingStore.get("mapLocal");
  mapLocal.value = mapLocalData ?? {
    toolEnabled: false,
    mapLocals: {}
  };
  state.mapLocal = mapLocal.value.toolEnabled ?? false;
};

onBeforeMount(() => {
  getTableData();
});

// 监听工具开关变化
watch(
  () => state.mapLocal,
  async (newValue) => {
    if (mapLocal.value) {
      mapLocal.value.toolEnabled = newValue;
      await settingStore.set("mapLocal", mapLocal.value);
    }
  }
);

watch(
  () => mapLocal.value?.mapLocals,
  (mapLocalRecord) => {
    if (mapLocalRecord) {
      checkboxArr.length = Object.keys(mapLocalRecord).length;
    } else {
      checkboxArr.length = 0;
    }

    if (!mapLocalRecord || Object.keys(mapLocalRecord).length === 0) {
      allSwitch.value = false;
      return;
    }

    let allEnabled = true;
    for (const key in mapLocalRecord) {
      if (!mapLocalRecord[key].enabled) {
        allEnabled = false;
        break;
      }
    }

    allSwitch.value = allEnabled;
  },
  {
    deep: 2,
    immediate: true
  }
);

// 处理导入
const importHandler = async () => {
  try {
    const path = await open({
      title: "导入",
      filters: [{ name: "xml", extensions: ["xml"] }],
      multiple: false
    });

    if (!path) return;
    // const importedMapLocal =
    // mapLocal.value = importedBreakpoints;
    const xmlObj: { mapLocal: MapLocal } = await readXmlFile(path);
    if (!xmlObj || !xmlObj.mapLocal) {
      message.error("导入失败：文件内容不合法");
      return;
    }
    const mapLocalData = xmlObj.mapLocal;
    const mapLocalImport: MapLocal = {
      toolEnabled: mapLocalData.toolEnabled,
      mapLocals: {}
    };
    for (const key in mapLocalData.mapLocals) {
      mapLocalImport.mapLocals[key] = mapLocalData.mapLocals[key];
    }
    mapLocal.value = mapLocalImport;
    console.log(mapLocalImport);
  } catch (e) {
    error("导入失败" + e);
    message.error(`导入失败：${e instanceof Error ? e.message : "未知错误"}`);
  }
};

const exportConfig = async (savePath: string, mapLocalData: MapLocal) => {
  const mapLocals = Object.values(mapLocalData.mapLocals);

  const xmlObj = {
    mapLocal: {
      toolEnabled: mapLocalData.toolEnabled,
      mapLocals
    }
  };

  try {
    if (savePath) {
      const hide = message.loading("导出配置中...", 0);
      await writeTextFile(savePath, createXmlStr(xmlObj));
      hide();
      message.success("导出配置成功");
    }
  } catch (e) {
    message.error("导出配置失败");
    error("导出配置失败" + e);
  }
};

// 处理导出
const exportHandler = async () => {
  try {
    const savePath = await save({
      filters: [{ name: "XML", extensions: ["xml"] }]
    });

    if (savePath && mapLocal.value) {
      await exportConfig(savePath, mapLocal.value);
    }
  } catch (e) {
    error("导出失败" + e);
    message.error(`导出失败：${e instanceof Error ? e.message : "未知错误"}`);
  }
};
</script>

<style scoped>
.mapLocalList {
  width: 100%;
  max-height: 300px; /* 设置固定高度 */
  overflow-y: auto; /* 垂直方向超出显示滚动条 */
}

.mapLocalList table {
  width: 100%;
  table-layout: fixed; /* 关键：使用固定表格布局 */
  border-collapse: collapse;
  border: 1px solid #e8e8e8;
}

.mapLocalList th,
.mapLocalList td {
  border: 1px solid #e8e8e8;
  padding: 8px;
  text-align: left;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mapLocalList th {
  background-color: #f2f2f2;
  font-weight: 400;
}

/* 可选：添加鼠标悬停效果 */
.mapLocalList tbody tr:hover {
  background-color: #fafafa;
}
</style>

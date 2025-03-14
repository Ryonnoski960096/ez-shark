<template>
  <Page>
    <p style="font-size: 13px">
      在请求和响应发送与接收之前对其进行拦截和编辑。
    </p>
    <!-- <p class="f-l f-g-10">
      <span>断点功能:</span>
      <Switch
        size="small"
        v-model:checked="state.breakpoint"
        checked-children="开"
        un-checked-children="关"
      />
    </p> -->
    <div class="breakpointList">
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
            <th>Location</th>
            <th>Breakpoint</th>
          </tr>
        </thead>
        <tbody v-if="Object.keys(breakpoints?.breakpoints || {}).length">
          <tr
            @click.right="oncontextmenu($event, item, key)"
            v-for="(item, key, index) in breakpoints?.breakpoints"
            :key="key"
          >
            <td>
              <Checkbox v-model:checked="checkboxArr[index]" />
            </td>
            <td>
              <!-- @change="breakpointsEnabled(item)" -->
              <Switch size="small" v-model:checked="item.enabled" />
            </td>
            <td style="width: 40%">
              {{ item.conditions.url }}
            </td>
            <td style="width: 20%">
              <span v-if="item.conditions.method">
                <Tag
                  :color="
                    item.conditions.method === `GET` ? `#2db7f5` : `#87d068`
                  "
                >
                  {{ item.conditions.method }}
                </Tag>
              </span>
              <span
                v-if="
                  !item.conditions.req_enable && !item.conditions.res_enable
                "
              >
                <Tooltip title="没有选择 Request 或 Response">
                  <Tag color="#f50"> 无效 </Tag>
                </Tooltip>
              </span>
              <span>
                <Tag v-if="item.conditions.req_enable" color="purple">
                  Request
                </Tag>
                <Tag v-if="item.conditions.res_enable" color="blue">
                  Response
                </Tag>
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <div style="margin-top: 20px" class="operation f-c f-g-20">
      <div class="f-c f-g-20">
        <Button @click="addBreakpoint" size="small">添加</Button>
      </div>
      <div class="f-c f-g-20">
        <Button @click="removeBreakpointList" size="small" danger>删除</Button>
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

<script setup lang="ts">
import { Reactive, reactive, ref, watch } from "vue";
import {
  Switch,
  Button,
  Checkbox,
  message,
  Tag,
  Tooltip
} from "ant-design-vue";
import Page from "@/components/Page.vue";
import { confirm, open, save } from "@tauri-apps/plugin-dialog";
import useBreakpointConfig from "@/hooks/useBreakpointConfig";
import { useSettingStore } from "@/stores/settings";
import { Breakpoint, Breakpoints } from "@/hooks/useBreakpointConfig";
import ContextMenu from "@imengyu/vue3-context-menu";
import { BreakpointEventName } from "@/enum/event-name";
import { removeBreakpoint, updateBreakpoint } from "@/api/breakpoint";
import { isSuccess } from "@/api";
import { windowInit, windowManager } from "@/stores/WindowManager";

// 窗口初始化
windowInit();

const WINDOW_CONFIG = {
  WIDTH: 400,
  HEIGHT: 320,
  TITLE: "断点编辑器"
};

const settingStore = useSettingStore();

const breakpoints = ref<Breakpoints | null>(null);

const newIds = ref<string[]>([]);

const getTableData = async () => {
  const breakpointList: Breakpoints | null | undefined =
    await settingStore.get("breakpoints");
  breakpoints.value = breakpointList ?? {
    toolEnabled: false,
    breakpoints: {}
  };
  console.log("breakpoints.value", breakpoints.value);
  state.breakpoint = breakpoints.value.toolEnabled ?? false;
};

getTableData();

const state = reactive({
  breakpoint: false
});

const allSwitch = ref(false);
const allChecked = ref(false);
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

// 全选功能
const handleAllSwitch = () => {
  if (!breakpoints.value?.breakpoints) return;

  const isCurrentlyAllChecked = allSwitch.value;
  console.log("allChecked", allSwitch.value);
  Object.keys(breakpoints.value.breakpoints).forEach((key) => {
    breakpoints.value!.breakpoints[key].enabled = isCurrentlyAllChecked;
  });

  settingStore.set("breakpoints", breakpoints.value);
};

const handleAllChecked = () => {
  if (!breakpoints.value?.breakpoints) return;

  // const isCurrentlyAllChecked = allChecked.value;
  for (let i = 0; i < checkboxArr.length; i++) {
    checkboxArr[i] = allChecked.value;
  }
};

watch(
  () => breakpoints.value?.breakpoints,
  (breakpointsRecord) => {
    if (breakpointsRecord) {
      checkboxArr.length = Object.keys(breakpointsRecord).length;
    } else {
      checkboxArr.length = 0;
    }
    console.log("checkboxArr", checkboxArr);

    if (!breakpointsRecord || Object.keys(breakpointsRecord).length === 0) {
      allSwitch.value = false;
      return;
    }

    let allEnabled = true;
    for (const key in breakpointsRecord) {
      if (!breakpointsRecord[key].enabled) {
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

// 监听工具开关变化
watch(
  () => state.breakpoint,
  async (newValue) => {
    if (breakpoints.value) {
      breakpoints.value.toolEnabled = newValue;
      await settingStore.set("breakpoints", breakpoints.value);
    }
  }
);

// 点击取消
const handleCancel = async () => {
  await windowManager.requestClose();
};

// 关闭前执行检查
async function handleCloseExamine() {
  console.log("newIds.value", newIds.value);
  if (newIds.value.length !== 0) {
    const res = await confirm("当前有未保存的配置，是否确认关闭？", {
      kind: "warning",
      okLabel: "确认",
      cancelLabel: "取消"
    });
    if (res) {
      for (const key in breakpoints.value!.breakpoints) {
        if (newIds.value.includes(key)) {
          delete breakpoints.value!.breakpoints[key];
        }
      }
      await settingStore.set("breakpoints", breakpoints.value);
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

const removeIds = ref<string[]>([]);

// 点击确定
const handleOk = async () => {
  const breakpointList = [];
  for (const key in breakpoints.value!.breakpoints) {
    breakpointList.push(breakpoints.value!.breakpoints[key]);
  }

  await updateBreakpoint(breakpointList);

  if (
    removeIds.value.length !== 0 &&
    !isSuccess(await removeBreakpoint(removeIds.value))
  ) {
    return message.error("删除配置失败");
  }

  await settingStore.set("breakpoints", breakpoints.value);
  windowManager.closeTasks.delete(handleCloseExamine);

  handleCancel();
};

const { importConfig, exportConfig, createBreakpoint } = useBreakpointConfig();

// 处理导入
const importHandler = async () => {
  try {
    const path = await open({
      title: "导入",
      filters: [{ name: "xml", extensions: ["xml"] }],
      multiple: false
    });

    if (!path) return;

    await importConfig(path, async (importedBreakpoints) => {
      await settingStore.set("breakpoints", importedBreakpoints);
      breakpoints.value = importedBreakpoints;
    });
  } catch (error) {
    console.error("导入失败", error);
    message.error(
      `导入失败：${error instanceof Error ? error.message : "未知错误"}`
    );
  }
};

// 处理导出
const exportHandler = async () => {
  try {
    const savePath = await save({
      filters: [{ name: "XML", extensions: ["xml"] }]
    });

    if (savePath && breakpoints.value) {
      await exportConfig(savePath, breakpoints.value);
    }
  } catch (error) {
    console.error("导出失败", error);
    message.error(
      `导出失败：${error instanceof Error ? error.message : "未知错误"}`
    );
  }
};

const openWindow = async (ops?: Record<string, any>) => {
  const [wvw] = await windowManager.createWindow(
    {
      url: "/breakpoint/edit",
      param: ops
    },
    {
      width: WINDOW_CONFIG.WIDTH,
      height: WINDOW_CONFIG.HEIGHT,
      title: WINDOW_CONFIG.TITLE
    }
  );

  // 监听事件
  const unListen = await wvw.listen(BreakpointEventName.SUBMIT, (event) => {
    try {
      const payload = event.payload as Breakpoint;
      console.log("data", payload);
      if (!breakpoints.value) {
        breakpoints.value = {
          toolEnabled: false,
          breakpoints: {}
        };
      }

      const [key, breakpoint] = createBreakpoint(payload);
      breakpoints.value.breakpoints[key] = breakpoint;
      console.log("breakpoints.value", breakpoints.value);

      settingStore.set("breakpoints", breakpoints.value);

      newIds.value.push(key);
      message.success("添加成功");
    } catch (error) {
      console.error(error);
      message.error(
        `添加失败：${error instanceof Error ? error.message : "未知错误"}`
      );
    } finally {
      unListen();
    }
  });
};

// 添加
const addBreakpoint = async () => {
  openWindow();
};

// 删除一个
const deleteBreakpoint = async (key: string) => {
  if (!breakpoints.value) return;

  delete breakpoints.value.breakpoints[key];
  removeIds.value.push(key);
};

// 批量删除
const removeBreakpointList = async () => {
  if (!breakpoints.value) return;

  const keys = Object.keys(breakpoints.value.breakpoints);
  console.log(keys);
  for (let i = 0; i < checkboxArr.length; i++) {
    if (checkboxArr[i]) {
      removeIds.value.push(keys[i]);
      checkboxArr[i] = false;
    }
  }
  removeIds.value.forEach((key) => {
    delete breakpoints.value?.breakpoints[key];
  });
};

// 编辑
const editBreakpoint = async (key: string) => {
  openWindow({ key });
};

// 右键菜单
function oncontextmenu(e: MouseEvent, _: Breakpoint, key: string) {
  e.preventDefault();

  ContextMenu.showContextMenu({
    x: e.x,
    y: e.y,
    items: [
      {
        label: "编辑",
        onClick: () => editBreakpoint(key)
      },
      {
        label: "删除",
        onClick: () => deleteBreakpoint(key)
      }
    ]
  });
}
</script>
<style scoped>
.breakpointList {
  width: 100%;
  max-height: 300px; /* 设置固定高度 */
  overflow-y: auto; /* 垂直方向超出显示滚动条 */
}

.breakpointList table {
  width: 100%;
  table-layout: fixed; /* 关键：使用固定表格布局 */
  border-collapse: collapse;
  border: 1px solid #e8e8e8;
}

.breakpointList th,
.breakpointList td {
  border: 1px solid #e8e8e8;
  padding: 8px;
  text-align: left;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.breakpointList th {
  background-color: #f2f2f2;
  font-weight: 400;
}

/* 可选：添加鼠标悬停效果 */
.breakpointList tbody tr:hover {
  background-color: #fafafa;
}
</style>

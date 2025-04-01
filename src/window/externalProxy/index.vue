<template>
  <div class="proxy-settings-container">
    <div class="main-content">
      <ElForm :model="proxyForm">
        <!-- 启用代理服务器 -->
        <ElFormItem size="small">
          <ElCheckbox v-model="proxyForm.enabled">
            <span class="text-lg">Use external proxy servers</span>
          </ElCheckbox>
        </ElFormItem>

        <!-- 代理配置部分 -->
        <ElCard class="mb-4">
          <div class="proxy-config-section">
            <!-- 左侧协议列表 -->
            <div class="protocol-list">
              <div class="protocol-title">Select a protocol to configure:</div>
              <div class="protocol-items">
                <ElRadioGroup
                  :disabled="disabled"
                  v-model="proxyForm.proxyType"
                >
                  <div
                    v-for="protocol in protocols"
                    :key="protocol.value"
                    class="protocol-item cp w"
                    :class="
                      protocol.value === configureActive
                        ? 'protocol-item-active'
                        : ''
                    "
                    @click="configureActive = protocol.value"
                  >
                    <ElRadio :value="protocol.value" />
                    {{ protocol.label }}
                  </div>
                </ElRadioGroup>
              </div>
            </div>

            <!-- 右侧配置详情 -->
            <div class="protocol-config" v-if="currentConfig">
              <ElForm label-width="auto" :model="currentConfig">
                <ElFormItem size="small" label="Server:">
                  <ElInput
                    size="small"
                    v-model="
                      currentConfig.mutableExternalProxyConfiguration.host
                    "
                    :disabled="disabled"
                    style="width: 230px"
                    placeholder="Host"
                  />
                  <span class="mx-2">Port:</span>
                  <ElInputNumber
                    size="small"
                    controls-position="right"
                    v-model="
                      currentConfig.mutableExternalProxyConfiguration.port
                    "
                    :min="1"
                    :disabled="disabled"
                    :max="65535"
                    style="width: 100px"
                  />
                </ElFormItem>

                <ElFormItem size="small">
                  <ElCheckbox
                    :disabled="disabled"
                    v-model="
                      currentConfig.mutableExternalProxyConfiguration
                        .requiresAuthentication
                    "
                  >
                    Proxy server requires authentication
                  </ElCheckbox>
                </ElFormItem>

                <div
                  :class="{
                    'form-disabled':
                      !currentConfig.mutableExternalProxyConfiguration
                        .requiresAuthentication
                  }"
                >
                  <ElFormItem size="small" label="Domain:">
                    <ElInput
                      v-model="
                        currentConfig.mutableExternalProxyConfiguration.domain
                      "
                      style="width: 380px"
                      :disabled="
                        !currentConfig.mutableExternalProxyConfiguration
                          .requiresAuthentication || disabled
                      "
                    />
                    <div class="text-gray-400 text-11px mt-1">
                      The domain field is only required for Windows
                      authentication (NTLM).
                    </div>
                  </ElFormItem>

                  <ElFormItem size="small" label="Username:">
                    <ElInput
                      v-model="
                        currentConfig.mutableExternalProxyConfiguration.username
                      "
                      style="width: 380px"
                      :disabled="
                        !currentConfig.mutableExternalProxyConfiguration
                          .requiresAuthentication || disabled
                      "
                    />
                  </ElFormItem>

                  <ElFormItem size="small" label="Password:">
                    <ElInput
                      v-model="
                        currentConfig.mutableExternalProxyConfiguration
                          .encryptedPassword
                      "
                      type="password"
                      style="width: 380px"
                      show-password
                      :disabled="
                        !currentConfig.mutableExternalProxyConfiguration
                          .requiresAuthentication || disabled
                      "
                    />
                  </ElFormItem>
                </div>
              </ElForm>
            </div>
          </div>
        </ElCard>

        <!-- Bypass 配置部分 -->
        <ElCard class="mb-4">
          <ElForm size="small">
            <ElFormItem
              label-position="top"
              label="Bypass external proxies for the following hosts:"
            >
              <ElInput
                type="textarea"
                v-model="bypassHosts"
                :rows="4"
                :disabled="disabled"
                placeholder="Enter hosts to bypass, one per line"
                style="width: 100%"
              />
            </ElFormItem>

            <ElFormItem>
              <ElCheckbox
                :disabled="disabled"
                v-model="proxyForm.alwaysBypassLocalhost"
              >
                Always bypass external proxies for localhost
              </ElCheckbox>
            </ElFormItem>
          </ElForm>
        </ElCard>
      </ElForm>
    </div>

    <!-- 底部操作按钮 -->
    <div class="footer">
      <div>
        <ElButton @click="importConfig">Import</ElButton>
        <ElButton @click="exportConfig">Export</ElButton>
      </div>
      <div>
        <ElButton type="primary" @click="saveConfig">Save</ElButton>
        <!-- <ElButton @click="showHelp">Help</ElButton> -->
        <ElButton @click="handleCancel">Cancel</ElButton>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, computed, onMounted, watch, toRaw } from "vue";
import {
  ElButton,
  ElCard,
  ElCheckbox,
  ElForm,
  ElFormItem,
  ElInput,
  ElInputNumber,
  ElRadioGroup,
  ElRadio
} from "element-plus";
import type { Entry } from "./model";
import { type ExternalProxy, ProxyType } from "./model";
import { windowInit, windowManager } from "@/stores/WindowManager";
import { commonIE, deepClone } from "@/utils/tools";
import { useImport } from "@/hooks";
import { cryptoService } from "@/utils/crypto";
import { useSettingStore } from "@/stores/settings";
import { exportXML } from "@/hooks/useExport";

interface Protocol {
  label: string;
  value: ProxyType;
  isChecked: boolean;
}

// configure选中状态
const configureActive = ref<ProxyType>(ProxyType.HTTP);

// 协议选项
const protocols = ref<Protocol[]>([
  {
    label: "Web Proxy (HTTP)",
    value: ProxyType.HTTP,
    isChecked: true
  },
  {
    label: "Secure Web Proxy (HTTPS)",
    value: ProxyType.HTTPS,
    isChecked: false
  },
  {
    label: "SOCKS Proxy",
    value: ProxyType.SOCKS,
    isChecked: false
  }
]);

watch(
  () => protocols.value.map((item) => item.isChecked),
  () => {
    const checked = protocols.value.filter((item) => item.isChecked);
    if (checked.length === 1) {
      configureActive.value = checked[0].value;
    }
  }
);

const disabled = computed(() => {
  return !proxyForm.value.enabled;
});

// 模拟数据
const proxyForm = ref<ExternalProxy>({
  configurations: {
    entry: [
      {
        string: ProxyType.HTTP,
        mutableExternalProxyConfiguration: {
          requiresAuthentication: false,
          host: "",
          port: 8080,
          domain: "",
          username: "",
          encryptedPassword: ""
        }
      },
      {
        string: ProxyType.HTTPS,
        mutableExternalProxyConfiguration: {
          requiresAuthentication: false,
          host: "",
          port: 443,
          domain: "",
          username: "",
          encryptedPassword: ""
        }
      },
      {
        string: ProxyType.SOCKS,
        mutableExternalProxyConfiguration: {
          requiresAuthentication: false,
          host: "",
          port: 1080,
          domain: "",
          username: "",
          encryptedPassword: ""
        }
      }
    ]
  },
  bypassDomains: {
    string: []
  },
  proxyType: ProxyType.HTTP,
  enabled: false,
  alwaysBypassLocalhost: true
});

// 当前选中协议的配置
const currentConfig = computed(() => {
  return proxyForm.value.configurations.entry.find(
    (config) => config.string === configureActive.value
  );
});

// bypass hosts
const bypassHosts = ref("");

watch(bypassHosts, (newVal) => {
  proxyForm.value.bypassDomains.string = newVal
    .split("\n")
    .filter((item) => item.trim() !== "");
});
const importTool = useImport();

// 方法
const importConfig = async () => {
  commonIE(async () => {
    const traffics = await importTool.importXmlFile();

    proxyForm.value = {
      ...traffics.externalProxy,
      bypassDomains: proxyForm.value.bypassDomains
    };

    proxyForm.value.configurations.entry.forEach((config: Entry) => {
      if (config.mutableExternalProxyConfiguration.encryptedPassword) {
        config.mutableExternalProxyConfiguration.encryptedPassword =
          cryptoService.decrypt(
            config.mutableExternalProxyConfiguration.encryptedPassword
          );
      }
    });
    bypassHosts.value = proxyForm.value.bypassDomains.string.join("\n");
  }, "导入");
};

const exportConfig = () => {
  commonIE(async () => {
    const data = deepClone(proxyForm.value);
    data.configurations.entry.forEach((config: Entry) => {
      if (config.mutableExternalProxyConfiguration.encryptedPassword) {
        config.mutableExternalProxyConfiguration.encryptedPassword =
          cryptoService.encrypt(
            config.mutableExternalProxyConfiguration.encryptedPassword
          );
      }
    });
    await exportXML(data);
  });
};

const settingStore = useSettingStore();

const saveConfig = async () => {
  const data = deepClone(toRaw(proxyForm.value));
  await Promise.all(
    data.configurations.entry.map(async (config: Entry) => {
      if (config.mutableExternalProxyConfiguration.encryptedPassword) {
        config.mutableExternalProxyConfiguration.encryptedPassword =
          cryptoService.encrypt(
            config.mutableExternalProxyConfiguration.encryptedPassword
          );
      }
    })
  );
  await settingStore.set("externalProxy", data);
  await windowManager.requestClose();
};

onMounted(async () => {
  const externalProxy = await settingStore.get<ExternalProxy>("externalProxy");
  if (!externalProxy) return;
  await Promise.all(
    externalProxy.configurations.entry.map(async (config: Entry) => {
      if (config.mutableExternalProxyConfiguration.encryptedPassword) {
        config.mutableExternalProxyConfiguration.encryptedPassword =
          cryptoService.decrypt(
            config.mutableExternalProxyConfiguration.encryptedPassword
          );
      }
    })
  );
  proxyForm.value = {
    ...externalProxy, // 先展开新数据
    bypassDomains: proxyForm.value.bypassDomains
  };

  bypassHosts.value = (externalProxy.bypassDomains.string as string[]).join(
    "\n"
  );
});
const handleCancel = async () => {
  await windowManager.requestClose();
};

// 窗口初始化
windowInit();
</script>

<style scoped>
.proxy-settings-container {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
  height: calc(100vh - 36px);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.header {
  margin-bottom: 20px;
}

.card-header {
  font-weight: bold;
  font-size: 14px;
}

.proxy-config-section {
  display: flex;
}

.protocol-list {
  min-width: 250px;
  border-right: 1px solid #eee;
  padding-right: 20px;
}

.protocol-title {
  margin-bottom: 15px;
  font-size: 14px;
}

.protocol-items {
  display: flex;
  flex-direction: column;
}

.protocol-item {
  padding: 4px 10px;
  border-radius: 4px;
  transition: background-color 0.3s;
  font-size: 14px;
  user-select: none;
}

.protocol-item-active {
  background-color: #cbcbcb;
}

.protocol-item:not(.protocol-item-active):hover {
  background-color: #f5f7fa;
}

.protocol-config {
  flex: 1;
  padding-left: 20px;
}

.footer {
  display: flex;
  gap: 10px;
  justify-content: space-between;
}

.mb-4 {
  margin-bottom: 16px;
}

.mx-2 {
  margin: 0 8px;
}

.text-lg {
  font-size: 16px;
}

.text-sm {
  font-size: 12px;
}

.text-gray-400 {
  color: #909399;
}

:deep(.el-checkbox__label) {
  font-size: 14px;
}

.form-disabled {
  opacity: 0.6;
  pointer-events: none;
}

/* 可选：如果你想让禁用状态的表单项有不同的背景色 */
:deep(.el-input.is-disabled .el-input__wrapper) {
  background-color: #f5f7fa;
}

:deep(.el-input-number.is-disabled .el-input__wrapper) {
  background-color: #f5f7fa;
}
</style>

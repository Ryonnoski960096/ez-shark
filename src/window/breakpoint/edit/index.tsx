import { defineComponent, ref } from "vue";
import useBreakpointConfig, {
  type Breakpoint
} from "@/hooks/useBreakpointConfig";
import {
  Form,
  Input,
  Select,
  Space,
  Button,
  message,
  Checkbox
} from "ant-design-vue";
import Page from "@/components/Page.vue";
import styles from "./breakpoint.module.css";
import { BreakpointEventName } from "@/enum/breakpoint";
import { windowInit, windowManager } from "@/stores/WindowManager";
import { error } from "@tauri-apps/plugin-log";

// 定义表单字段配置
const formConfig = [
  {
    label: "Method",
    key: "conditions.method",
    type: "select",
    options: [
      { value: undefined, label: "请选择" },
      { value: "GET", label: "GET" },
      { value: "POST", label: "POST" }
    ],
    style: { width: "120px" }
  },
  {
    label: "Url",
    key: "conditions.url",
    type: "input"
  },
  {
    label: "Request Header",
    key: "conditions.request.header",
    type: "input"
  },
  {
    label: "Request Body",
    key: "conditions.request.body",
    type: "input"
  },
  {
    label: "Response Header",
    key: "conditions.response.header",
    type: "input"
  },
  {
    label: "Response Body",
    key: "conditions.response.body",
    type: "input"
  }
];

export default defineComponent({
  name: "BreakpointConfig",
  setup() {
    const prams = windowInit();
    delete prams.parentWindowId;

    // 使用 ref 存储表单数据
    const formData = ref<Breakpoint>({
      id: undefined,
      enabled: true,
      conditions: {
        req_enable: false,
        res_enable: false,
        url: undefined,
        method: undefined,
        request: {
          header: undefined,
          body: undefined
        },
        response: {
          header: undefined,
          body: undefined
        }
      }
    });

    if (prams.key) {
      const breakpointConfig = useBreakpointConfig();
      const res = breakpointConfig.getBreakpointByKey(prams.key as string);
      if (res) {
        formData.value = res;
      }
    }

    // 通用的值获取器
    const getNestedValue = (obj: any, path: string) => {
      return path.split(".").reduce((acc, part) => acc?.[part], obj);
    };

    // 通用的值设置器
    const setNestedValue = (obj: any, path: string, value: any) => {
      const parts = path.split(".");
      const lastKey = parts.pop()!;
      const target = parts.reduce((acc, part) => acc[part], obj);
      target[lastKey] = value;
    };

    // 渲染表单项
    const renderFormItem = (field: any) => {
      // 处理 Select 类型
      if (field.type === "select") {
        return (
          <Select
            style={field.style || {}}
            size="small"
            value={getNestedValue(formData.value, field.key)}
            onUpdate:value={(val) =>
              setNestedValue(formData.value, field.key, val)
            }
            allowClear
          >
            {field.options.map((option: any) => (
              <Select.Option key={option.value} value={option.value}>
                {option.label}
              </Select.Option>
            ))}
          </Select>
        );
      }

      // 处理 Input 类型
      return (
        <Input
          size="small"
          value={getNestedValue(formData.value, field.key)}
          onUpdate:value={(val) =>
            setNestedValue(formData.value, field.key, val)
          }
          allowClear
        />
      );
    };

    // 提交处理
    const handleSubmit = async (e: Event) => {
      e.preventDefault();

      try {
        const conditions = formData.value.conditions;

        if (!conditions.url && !conditions.method) {
          if (!conditions.request?.header && conditions.request?.body)
            throw new Error("未填写请求头或url或请求方法");

          if (!conditions.response?.header && conditions.response?.body)
            throw new Error("未填写响应头或url或请求方法");

          if (
            !conditions.request?.header &&
            !conditions.request?.body &&
            !conditions.response?.header &&
            !conditions.response?.body
          )
            throw new Error("请先填写断点条件");
        }

        // 发送数据到父窗口
        await windowManager.window.emit(
          BreakpointEventName.SUBMIT,
          formData.value
        );

        // 关闭窗口
        await windowManager.requestClose();
      } catch (errorInfo) {
        error("表单验证失败:" + errorInfo);
        message.error((errorInfo as Error).message);
      }
    };

    // 取消处理
    const handleCancel = async () => {
      // 重置表单
      formData.value = {
        id: undefined,
        enabled: true,
        conditions: {
          req_enable: false,
          res_enable: false,
          url: undefined,
          method: undefined,
          request: {
            header: undefined,
            body: undefined
          },
          response: {
            header: undefined,
            body: undefined
          }
        }
      };
      await windowManager.requestClose();
    };

    return () => (
      <Page class={styles.breakpoint}>
        <div class={styles.formContainer}>
          <Form
            onSubmit={handleSubmit}
            layout="horizontal"
            model={formData.value}
            class={styles.customForm}
          >
            <table class={styles.formTable}>
              <tbody>
                {formConfig.map((field) => (
                  <tr class={styles.formRow} key={field.key}>
                    <td class={styles.labelCell}>{field.label}</td>
                    <td class={styles.controlCell}>{renderFormItem(field)}</td>
                  </tr>
                ))}
                <tr>
                  <td class="f-r">
                    <div class="response-group">
                      <span>Request </span>
                      <Checkbox
                        checked={formData.value.conditions.req_enable}
                        onUpdate:checked={(checked) => {
                          formData.value.conditions.req_enable = checked;
                        }}
                      />
                    </div>
                  </td>
                  <td>
                    <div class="response-group">
                      &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
                      <span>Response </span>
                      <Checkbox
                        checked={formData.value.conditions.res_enable}
                        onUpdate:checked={(checked) => {
                          formData.value.conditions.res_enable = checked;
                        }}
                      />
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>

            <Space class={[styles.formActions, "mt-5px"]}>
              <Button html-type="submit" size="small" type="primary">
                提交
              </Button>
              <Button size="small" onClick={handleCancel}>
                取消
              </Button>
            </Space>
          </Form>
        </div>
      </Page>
    );
  }
});

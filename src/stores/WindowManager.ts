// 重新整理思路
// 重新设计调度器
import { WindowEvent } from "@/enum/window";
import useUrlParams from "@/hooks/useUrlParams";
import { WebviewOptions } from "@tauri-apps/api/webview";
import {
  WebviewWindow,
  getAllWebviewWindows,
  getCurrentWebviewWindow
} from "@tauri-apps/api/webviewWindow";
import { WindowOptions } from "@tauri-apps/api/window";
import { onMounted } from "vue";

export type webviewOps = Omit<WebviewOptions, "x" | "y" | "width" | "height"> &
  WindowOptions;

// 辅助方法
const urlParams = useUrlParams();

/**
 * 创建新的窗口key
 * 使用示例
 * const windowKey1 = createNewWindowKey("/path/to/window");
 * 输出: '/path/to/window?'
 *
 * const windowKey2 = createNewWindowKey("/path/to/window", {
 * id: 123,
 * name: "John Doe",
 * extra: { data: "some data" },
 * });
 * 输出: '/path/to/window?id=123&name=John%20Doe&extra=%7B%22data%22%3A%22some%20data%22%7D'
 * @param str
 * @param param
 * @returns
 */
export const createNewWindowKey = (
  str: string,
  param: Record<string, any> = {}
): string => {
  // 如果没有参数，直接返回原字符串
  if (Object.keys(param).length === 0) {
    return `${str}?`;
  }

  const queryString = urlParams.createUrlParams(param);

  // 组合最终的窗口key
  return `${str}?${queryString}`;
};

export function createSafeWindowLabel(url: string): string {
  // 移除特殊字符，替换为下划线
  const safeLabel = url
    .replace(/[^a-zA-Z0-9-/:]/g, "_") // 替换不允许的字符
    .replace(/[.]/g, "_") // 将点替换为下划线
    .replace(/[/]/g, "-") // 将斜杠替换为横线
    .substring(0, 50); // 限制长度

  return `child_${safeLabel}`;
}

export interface urlParams {
  url: string;
  param?: Record<string, any>;
}

// 原问题：
// 使用 parent 会导致任务栏只有一个窗口不利于管理
// 不使用 parent 会在父窗口关闭时不能关闭子窗口
// 点击右上角的关闭是销毁，无法触发close事件监听
// 窗口管理方案不够灵活

// 设计思路：
// 关闭原生的关闭按钮，自定义关闭按钮，用于处理窗口关闭
// 不由父窗口管理子窗口，由主窗口管理所有窗口
export class WindowManager {
  public windows: Map<
    string,
    {
      WebviewWindow: WebviewWindow;
      parentWindowLabel: string | undefined;
      childrenWindows: Set<WebviewWindow>;
    }
  > = new Map();

  public window = getCurrentWebviewWindow();

  /**
   * 关闭窗口前执行的任务
   * 必须返回true或false
   * true: 允许关闭
   * false: 不允许关闭
   */
  public closeTasks: Set<() => Promise<boolean>> = new Set();

  /**
   * 是否是主窗口
   * @returns
   */
  public isMainWindow() {
    return this.window.label === "main";
  }

  /**
   * 根据label获取窗口
   * @param label
   * @returns
   */
  public async getWebviewWindowByLabel(label: string) {
    let webviewWindow: WebviewWindow | null = null;
    webviewWindow = this.windows.get(label)?.WebviewWindow ?? null;
    if (webviewWindow) return webviewWindow;
    for (const wvw of await getAllWebviewWindows()) {
      if (wvw.label === label) {
        webviewWindow = wvw;
        break;
      }
    }
    if (webviewWindow) return webviewWindow;
    return null;
  }

  /**
   * 运行关闭任务
   * 如果有一个任务返回false或报错，那么就不关闭窗口
   * @returns
   */
  private async runCloseTasks() {
    while (this.closeTasks.size > 0) {
      const task = this.closeTasks.values().next().value; // 获取第一个任务

      if (task) {
        try {
          const result = await task();
          if (result === false) {
            return false;
          }
        } catch (error) {
          return false;
        }
      }
    }
    return true;
  }

  /**
   * 构造函数
   */
  public constructor() {
    // 初始化主窗口
    if (this.isMainWindow()) {
      this.windows.set("main", {
        WebviewWindow: this.window,
        parentWindowLabel: undefined,
        childrenWindows: new Set()
      });

      // 监听窗口创建事件，完成关联
      this.window.listen(WindowEvent.CREATE, async (event) => {
        const payload = event.payload as {
          label: string;
          parentLabel: string;
        };
        console.log("窗口创建", payload);
        if (!payload) return;
        const parentWebviewWindow = this.windows.get(payload.parentLabel);
        const childrenWindows = new Set<WebviewWindow>();

        let webviewWindow: WebviewWindow | null =
          await this.getWebviewWindowByLabel(payload.label);

        if (!webviewWindow) return;

        // 添加子窗口到父窗口的子窗口列表中
        if (parentWebviewWindow) {
          parentWebviewWindow.childrenWindows.add(webviewWindow);

          this.windows.set(payload.parentLabel, {
            ...parentWebviewWindow
          });
        }

        // 添加窗口到窗口管理器中
        if (!this.windows.has(payload.label)) {
          this.windows.set(payload.label, {
            parentWindowLabel: payload.parentLabel,
            WebviewWindow: webviewWindow,
            childrenWindows
          });
        }
      });

      // 监听窗口关闭事件，删除关联
      this.window.listen(WindowEvent.CLOSE_REQUESTED, async (event) => {
        // this.window.emit(WindowEvent.CLOSE, payload.label);

        if (!event.payload) return;
        const payload = event.payload as string;

        const window = this.windows.get(payload);
        console.log("关闭窗口", this.windows, payload);
        if (!window) return;

        const webviewWindow = this.windows.get(payload);
        const childrenWindows = webviewWindow?.childrenWindows;

        // 关闭所有子窗口
        if (childrenWindows) {
          childrenWindows.forEach((childWindow) => {
            childWindow.close();
            this.windows.delete(childWindow.label);
          });
        }

        // 关闭窗口
        await window.WebviewWindow.close();

        // 删除父窗口的子窗口关联关系
        if (webviewWindow?.parentWindowLabel) {
          const parentWVW = this.windows.get(webviewWindow.parentWindowLabel);
          parentWVW?.childrenWindows.delete(webviewWindow.WebviewWindow);
        }

        // 删除窗口
        this.windows.delete(payload);
      });
    }
  }

  /**
   * 创建窗口
   * @param urlParams
   * @param ops
   * @returns [WebviewWindow, windowLabel, url, isNew]
   */
  public async createWindow(
    urlParams: urlParams,
    ops?: webviewOps
  ): Promise<[WebviewWindow, string, string, boolean]> {
    if (!urlParams.param) urlParams.param = {};

    const [windowLabel, url] = this.createWindowLabel(urlParams);

    // 如果窗口已经存在，则直接返回
    if (this.windows.has(windowLabel)) {
      const WVW = this.windows.get(windowLabel)!.WebviewWindow;
      WVW?.setFocus();
      return [WVW, windowLabel, url, false];
    }

    const childWindowDefaults = {
      decorations: false, // 是否显示窗口装饰(如标题栏、边框等)。true表示显示，false则创建无边框窗口
      skipTaskbar: false, // 是否在任务栏中显示窗口。false表示在任务栏显示，true则不显示
      // alwaysOnTop: true, // 是否始终保持窗口在最前面。true表示窗口将始终位于其他窗口之上
      center: true, // 是否将窗口居中显示。true表示在屏幕中央打开窗口
      focus: true, // 是否在创建时自动获得焦点。true表示窗口创建后立即获得焦点
      shadow: true, // 是否为窗口添加阴影效果。true表示添加阴影，增强视觉效果
      visible: true, // 是否在创建时立即显示窗口。true表示创建后立即可见
      maximizable: false, // 是否允许窗口最大化。false表示禁用最大化按钮
      minimizable: false, // 是否允许窗口最小化。false表示禁用最小化按钮
      closable: false // 是否允许通过系统UI关闭窗口。false表示禁用关闭按钮
    };

    const windowOptions: webviewOps = {
      url,
      ...childWindowDefaults,
      ...ops
    };
    // 创建窗口
    return [
      new WebviewWindow(windowLabel, windowOptions),
      windowLabel,
      url,
      true
    ];
  }

  /**
   * 创建label
   */
  public createWindowLabel(urlParams: urlParams) {
    const parentWindowLabel = this.window.label;
    urlParams.param = {
      ...urlParams.param,
      parentWindowId: parentWindowLabel
    };

    const url = createNewWindowKey(urlParams.url, urlParams.param);

    const windowLabel = createSafeWindowLabel(url);

    return [windowLabel, url];
  }

  /**
   * 窗口是否存在
   */
  public hasWindow(urlParams: urlParams) {
    const [windowLabel] = this.createWindowLabel(urlParams);
    return this.windows.has(windowLabel);
  }

  /**
   * 请求关闭当前窗口
   */
  public async requestClose(wvw: WebviewWindow = this.window) {
    if (!(await this.runCloseTasks())) return;

    return await wvw.emitTo("main", WindowEvent.CLOSE_REQUESTED, wvw.label);
  }
}

export const windowManager = new WindowManager();
/**
 * 初始化窗口，子窗口必须且只能在vue文件中执行这个方法一次
 * 否则无法建立关联关系
 * @returns
 */
export const windowInit = () => {
  const webview = getCurrentWebviewWindow();

  const prams = urlParams.parseUrlParams();
  const parentWindowId = prams.parentWindowId as string;
  onMounted(() => {
    if (!windowManager.isMainWindow()) {
      webview.emitTo("main", WindowEvent.CREATE, {
        label: webview.label,
        parentLabel: parentWindowId as string
      });
    }
  });
  return prams;
};

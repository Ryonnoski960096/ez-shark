import type { ITrafficDataDetail } from "@/stores/traffic";
import { error } from "@tauri-apps/plugin-log";

// 事件处理函数类型
type EventHandler<T = any> = (payload: T) => void;

// 事件映射接口，限制 key 只能是 string
interface EventMap {
  [eventName: string]: any;
}

class EventBus<T extends EventMap = EventMap> {
  // 修改 handlers 的类型定义，明确指定 key 为 string
  private handlers: Map<string, Set<EventHandler>> = new Map();

  private static instance: EventBus | null = null;

  private constructor() {
    this.handlers = new Map();
  }

  public static getInstance<T extends EventMap>(): EventBus<T> {
    if (!this.instance) {
      this.instance = new EventBus<T>();
    }
    return this.instance as EventBus<T>;
  }

  /**
   * 订阅事件
   */
  public on<K extends keyof T & string>(
    event: K,
    handler: EventHandler<T[K]>
  ): void {
    if (!this.handlers.has(event)) {
      this.handlers.set(event, new Set());
    }
    this.handlers.get(event)!.add(handler);
  }

  /**
   * 取消订阅事件
   */
  public off<K extends keyof T & string>(
    event: K,
    handler: EventHandler<T[K]>
  ): void {
    if (this.handlers.has(event)) {
      this.handlers.get(event)!.delete(handler);
      if (this.handlers.get(event)!.size === 0) {
        this.handlers.delete(event);
      }
    }
  }

  /**
   * 只订阅一次事件
   */
  public once<K extends keyof T & string>(
    event: K,
    handler: EventHandler<T[K]>
  ): void {
    const onceHandler: EventHandler<T[K]> = (payload) => {
      handler(payload);
      this.off(event, onceHandler);
    };
    this.on(event, onceHandler);
  }

  /**
   * 发布事件
   */
  public emit<K extends keyof T & string>(event: K, payload: T[K]): void {
    if (this.handlers.has(event)) {
      this.handlers.get(event)!.forEach((handler) => {
        try {
          handler(payload);
        } catch (e) {
          error(`Error in event handler for ${String(event)}:${e}`);
        }
      });
    }
  }

  /**
   * 清除所有事件监听
   */
  public clear(): void {
    this.handlers.clear();
  }

  /**
   * 获取某个事件的所有监听器数量
   */
  public listenerCount<K extends keyof T & string>(event: K): number {
    return this.handlers.get(event)?.size || 0;
  }
}

export interface ActiveTraffic {
  id: string;
  keyword: string;
  position: string;
  index: number;
  method?: "response" | "request";
  sessionId: string;
}

// 定义具体的事件类型映射
export interface AppEventMap {
  "change:trafficDetail": ITrafficDataDetail | null;
  "change:breakpointData": ITrafficDataDetail;
}

// 导出单例实例
export const eventBus = EventBus.getInstance<AppEventMap>();

// 导出类型
export type { EventHandler };

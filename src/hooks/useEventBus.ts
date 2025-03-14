import { onUnmounted } from "vue";
import {
  eventBus,
  type AppEventMap,
  type EventHandler
} from "@/utils/eventBus";

export default function useEventBus() {
  const listeners = new Map<keyof AppEventMap, EventHandler[]>();

  const on = <K extends keyof AppEventMap>(
    event: K,
    handler: EventHandler<AppEventMap[K]>
  ) => {
    eventBus.on(event, handler);

    if (!listeners.has(event)) {
      listeners.set(event, []);
    }
    listeners.get(event)!.push(handler);
  };

  const emit = <K extends keyof AppEventMap>(
    event: K,
    payload: AppEventMap[K]
  ) => {
    eventBus.emit(event, payload);
  };

  onUnmounted(() => {
    listeners.forEach((handlers, event) => {
      handlers.forEach((handler) => {
        eventBus.off(event, handler);
      });
    });
    listeners.clear();
  });

  return {
    on,
    emit,
    once: eventBus.once.bind(eventBus),
    off: eventBus.off.bind(eventBus)
  };
}

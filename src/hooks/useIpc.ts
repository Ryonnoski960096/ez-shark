import type { Event} from "@tauri-apps/api/event";
import { listen as tauriListen } from "@tauri-apps/api/event";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";

export interface IpcData<T> {
  code: number;
  message: string;
  data: T;
}

export default function useIpc() {
  const listen = async <T>(event: string, handler: (data: T) => void) => {
    return await tauriListen<T>(event, (event: Event<T>) => {
      handler(event.payload as T);
    });
  };

  const invoke = async <T>(cmd: string, params?: any): Promise<T> => {
    return await tauriInvoke<T>(cmd, params);
  };

  return {
    listen,
    invoke,
  };
}

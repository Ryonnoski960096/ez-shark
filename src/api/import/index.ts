import { useIpc } from "@/hooks";
import type { TrafficData } from "@/stores/traffic";

const ipc = useIpc();

export function importSessionApi(path: string, sessionId: string) {
  return ipc.invoke<TrafficData[]>("import_session", {
    path,
    sessionId
  });
}

export function importCharlesApi(path: string, sessionId: string) {
  return ipc.invoke<TrafficData[]>("import_charles", {
    path,
    sessionId
  });
}

export function importHARApi(path: string, sessionId: string) {
  return ipc.invoke<TrafficData[]>("import_har", {
    path,
    sessionId
  });
}

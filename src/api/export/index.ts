import { useIpc } from "@/hooks";
import type { FormatType } from "./model";
import { useSessionStore } from "@/stores/session";

const ipc = useIpc();

export function exportApi(path: string, sessionId: string) {
  return ipc.invoke("handle_export_traffic", {
    path,
    sessionId
  });
}

export function copyApi(id: number, format: FormatType) {
  const sessionStore = useSessionStore();
  return ipc.invoke<string>("handle_copy_traffic", {
    id,
    format,
    sessionId: sessionStore.currentSession ?? ""
  });
}

import { useIpc } from "@/hooks";
import { FormatType } from "./model";

const ipc = useIpc();

export function exportApi(path: string) {
  return ipc.invoke("handle_export_traffic", {
    path
  });
}

export function copyApi(id: number, format: FormatType) {
  return ipc.invoke<string>("handle_copy_traffic", {
    id,
    format
  });
}

import { useIpc } from "@/hooks";
import { TrafficData } from "@/stores/traffic";

const ipc = useIpc();

export function importSessionApi(path: string) {
  return ipc.invoke<TrafficData[]>("import_session", {
    path
  });
}

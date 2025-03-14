import { useIpc } from "@/hooks";

const ipc = useIpc();

export function installCertificate() {
  return ipc.invoke("open_config_dir");
}

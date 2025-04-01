import { useIpc } from "@/hooks";
const ipc = useIpc();

/**
 * 检查charles是否运行中
 * @returns
 */
export function isCharlesRunning(): Promise<boolean> {
  return ipc.invoke("is_charles_running");
}

/**
 * 终止charles
 * @returns
 */
export function killCharles(): Promise<boolean> {
  return ipc.invoke("kill_charles");
}

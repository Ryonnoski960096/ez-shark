import { useIpc } from "@/hooks";
import { Response } from "../model";

const ipc = useIpc();

/**
 * 设置端口
 * @param breakpointList
 * @returns
 */
export function setPort(port: number): Promise<Response> {
  return ipc.invoke("setting_port", {
    port
  });
}

/**
 * 改变流量处理状态
 * @param monitor_traffic
 * @returns
 */
export function changeMonitorTraffic(
  monitorTraffic: boolean
): Promise<Response> {
  return ipc.invoke("change_monitor_traffic", {
    monitorTraffic
  });
}
